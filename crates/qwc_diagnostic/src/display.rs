use std::{collections::BTreeMap, fmt, path::Path};
use owo_colors::OwoColorize;
use qwc_arena::Files;
use crate::{Level, Message};


pub struct MessageDisplay<'a>(&'a Message, &'a Files);

impl Message {
  pub fn display<'a>(&'a self, far: &'a Files) -> MessageDisplay<'a> {
    MessageDisplay(self, far)
  }
}

impl<'a> fmt::Display for MessageDisplay<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let MessageDisplay(msg, far) = self;

    // level[code]: message
    if let Some(code) = &msg.message.code {
      writeln!(f, "{}{}E{:04x}{}{} {}", msg.level, "[".bright_black(), code.0, "]".bright_black(), ":".bright_black(), msg.message.msg)?;
    } else {
      writeln!(f, "{}{} {}", msg.level, ":".bright_black(), msg.message.msg)?;
    }

    if msg.labels.is_empty() { return Ok(()) }

    // Global satır basamağı ve dosya bazında etiket gruplama
    let mut max_line = 1u32;
    let mut file_groups: BTreeMap<u16, Vec<(usize, &crate::Label)>> = BTreeMap::new();

    for (idx, label) in msg.labels.iter().enumerate() {
      let hr = label.span.interval(far);
      let s = hr.start[0].get() as u32;
      let e = hr.end[0].get() as u32;
      max_line = max_line.max(s).max(e);
      file_groups.entry(label.span.fid).or_default().push((idx, label));
    }

    let padd = std::cmp::max(max_line.ilog10() +1, 1) as usize;
    let space = " ".repeat(padd);

    // Her dosya bloğunu sırayla çiz
    for (idx, (fid, labels)) in file_groups.into_iter().enumerate() {
      let file_ref = far.get(fid);
      let file_content = match std::str::from_utf8(file_ref.map()) {
        Ok(c) => c,
        Err(_) => continue,
      };

      // İlk dosya için -->, sonraki dosya geçişleri için :::
      let marker = if idx == 0 { "-->".bright_black().to_string() } else { ":::".bright_black().to_string() };
      let first_hr = labels[0].1.span.interval(far);
      let display_path = normalize_path(file_ref.fpath());

      writeln!(f, "{} {marker} {}{}{}{}{}", space, display_path.blue().bold(), ":".bright_black(), first_hr.start[0], ":".bright_black(), first_hr.start[1])?;
      writeln!(f, " {} {}", space, "|".bright_black())?;

      render_file_snippets(f, file_content, labels, padd, far)?;
    }
    
    // Notes
    for note in &msg.notes {
      writeln!(f, " {} {} {}{} {}", " ".repeat(padd), "=".bright_black(), "note".blue().bold(), ":".bright_black(), note)?;
    }

    // Explain
    if let Some(code) = &msg.message.code {
      writeln!(f, " {} {} {}{} for more information, try `qw explain E{:04x}`", " ".repeat(padd), "?".bright_black(), "help".blue().bold(), ":".bright_black(), code.0)?;
    }

    Ok(())
  }
}

impl fmt::Display for Level {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Level::Fatal => write!(f, "{}", "fatal".red().bold()),
      Level::Error => write!(f, "{}", "error".red().bold()),
      Level::Warn  => write!(f, "{}", "warn".yellow().bold()),
      Level::Hint  => write!(f, "{}", "hint".yellow().bold()),
    }
  }
}

struct LineRef<'s> {
  start_byte: usize,
  text: &'s str,
}

fn render_file_snippets(f: &mut fmt::Formatter<'_>, content: &str, labels: Vec<(usize, &crate::Label)>, padd: usize, far: &Files) -> fmt::Result {
  let space = " ".repeat(padd);

  let mut lines: Vec<LineRef<'_>> = Vec::new();
  let mut offset = 0;
  for slice in content.split('\n') {
    let clean = slice.trim_end_matches('\r');
    lines.push(LineRef {
      start_byte: offset,
      text: clean,
    });
    offset += slice.len() +1;
  }

  let mut sorted_labels = labels;
  sorted_labels.sort_by_key(|(_, l)| l.span.range().start);

  let mut intervals: Vec<(u32, u32)> = sorted_labels
    .iter()
    .map(|(_, l)| {
      let hr = l.span.interval(far);
      (hr.start[0].get() as u32, hr.end[0].get() as u32)
    })
    .collect();
  intervals.sort_by_key(|(s, _)| *s);

  let mut merged_intervals: Vec<(u32, u32)> = vec![];
  for (start, end) in intervals {
    if let Some(last) = merged_intervals.last_mut() {
      if start <= last.1 + 1 + 2 {
        last.1 = last.1.max(end);
      } else {
        merged_intervals.push((start, end));
      }
    } else {
      merged_intervals.push((start, end));
    }
  }

  for (b_idx, (b_start, b_end)) in merged_intervals.into_iter().enumerate() {
    if b_idx > 0 {
      writeln!(f, " {} {}", " ".repeat(padd), ":".bright_black())?;
    }

    for line_no in b_start..=b_end {
      let line_idx = (line_no - 1) as usize;
      let line = match lines.get(line_idx) {
        Some(l) => l,
        None => continue,
      };

      let line_str = line_no.to_string();
      let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));
      writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), line.text)?;

      // Bu satıra denk gelen tüm etiketlerin alt çizgilerini bas
      for (orig_idx, label) in &sorted_labels {
        let rng = label.span.range();
        // İlk eklenen etiket birincil (^), sonrakiler ikincil (-)
        let is_primary = *orig_idx == 0;

        if let Some(underline) = build_underline(line.text, line.start_byte, rng.start, rng.end, is_primary) {
          let (colored_underline, colored_msg) = color_underline(&underline, label.message.as_deref(), is_primary);
          if colored_msg.is_empty() {
            writeln!(f, " {} {} {}", space, "|".bright_black(), colored_underline)?;
          } else {
            writeln!(f, " {} {} {} {}", space, "|".bright_black(), colored_underline, colored_msg)?;
          }
        }
      }
    }
  }

  writeln!(f, " {} {}", space, "|".bright_black())?;
  Ok(())
}

fn build_underline(line_text: &str, line_byte_start: usize, span_start: usize, span_end: usize, is_primary: bool) -> Option<String> {
  let line_byte_end = line_byte_start + line_text.len();
  let is_point_span = span_start == span_end;

  if is_point_span {
    if span_start < line_byte_start || span_start > line_byte_end {
      return None;
    }
  } else if span_end <= line_byte_start || span_start >= line_byte_end {
    return None;
  }

  let marker = if is_primary { '^' } else { '-' };
  let mut underline = String::new();
  let mut has_marker = false;

  for (char_offset, ch) in line_text.char_indices() {
    let global_char_pos = line_byte_start + char_offset;

    // Çizgi basımı tamamlandıysa satırın geri kalanı için boşluk üretme, hemen çık
    if has_marker && global_char_pos >= span_end {
      break;
    }

    let should_mark = if is_point_span {
      global_char_pos == span_start
    } else {
      global_char_pos >= span_start && global_char_pos < span_end
    };

    if should_mark {
      underline.push(marker);
      has_marker = true;
      
      if is_point_span { break }
    } else if ch == '\t' {
      underline.push('\t');
    } else {
      underline.push(' ');
    }
  }

  // Noktasal hata tam satır sonuna denk geliyorsa en sona 1 adet ekle
  if is_point_span && span_start >= line_byte_end && !has_marker {
    underline.push(marker);
    has_marker = true;
  }

  if has_marker {
    Some(underline)
  } else {
    None
  }
}

fn color_underline(underline: &str, msg: Option<&str>, is_primary: bool) -> (String, String) {
  if is_primary {
    (
      underline.yellow().bold().to_string(),
      msg.map(|m| m.yellow().bold().to_string()).unwrap_or_default(),
    )
  } else {
    (
      underline.bright_cyan().bold().to_string(),
      msg.map(|m| m.bright_cyan().bold().to_string()).unwrap_or_default(),
    )
  }
}

fn normalize_path(raw_path: &str) -> String {
  let path = Path::new(raw_path);
  let path = path.strip_prefix(".").unwrap_or(path);

  if let Ok(cwd) = std::env::current_dir() {
    path.strip_prefix(&cwd).unwrap_or(path).display().to_string()
  } else {
    path.display().to_string()
  }
}
