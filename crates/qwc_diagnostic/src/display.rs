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
      writeln!(f, "{}{}E{:04x}{}{} {}", msg.level, "[".bright_black(), code, "]".bright_black(), ":".bright_black(), msg.message.msg)?;
    } else {
      writeln!(f, "{}{} {}", msg.level, ":".bright_black(), msg.message.msg)?;
    }

    if msg.labels.is_empty() && msg.suggestions.is_empty() && msg.notes.is_empty() { return Ok(()) }

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

    for sugg in &msg.suggestions {
      for label in &sugg.labels {
        let hr = label.span.interval(far);
        let s = hr.start[0].get() as u32;
        let e = hr.end[0].get() as u32;
        max_line = max_line.max(s).max(e);
      }
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
    
    // Suggestions
    for sugg in &msg.suggestions {
      writeln!(f, " {} {} {}{}{}{} {}", space, "=".bright_black(), "suggestion".blue().bold(), "(".bright_black(), sugg.applicability.bright_black(), "):".bright_black(), sugg.message.msg)?;
      
      // Group labels of this suggestion by file
      let mut file_groups: BTreeMap<u16, Vec<(usize, &crate::Label)>> = BTreeMap::new();
      for (idx, label) in sugg.labels.iter().enumerate() {
        file_groups.entry(label.span.fid).or_default().push((idx, label));
      }

      for (fid, labels) in file_groups {
        let file_ref = far.get(fid);
        let file_content = match std::str::from_utf8(file_ref.map()) {
          Ok(c) => c,
          Err(_) => continue,
        };

        let is_different_file = msg.labels.first().map(|l| l.span.fid != fid).unwrap_or(true);
        if is_different_file {
          let first_hr = labels[0].1.span.interval(far);
          let display_path = normalize_path(file_ref.fpath());
          writeln!(f, "{} {} {}{}{}{}{}", space, ":::".bright_black(), display_path.blue().bold(), ":".bright_black(), first_hr.start[0], ":".bright_black(), first_hr.start[1])?;
        }
        writeln!(f, " {} {}", space, "|".bright_black())?;

        render_single_suggestion(f, file_content, sugg, labels, padd, far)?;
      }
    }

    // Explain
    if let Some(code) = &msg.message.code {
      writeln!(f, " {} {} {}{} for more information, try `qw explain E{:04x}`", " ".repeat(padd), "?".bright_black(), "help".blue().bold(), ":".bright_black(), code)?;
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
      writeln!(f, " {}{}", " ".repeat(padd-1), "...".bright_black())?;
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

fn render_single_suggestion(f: &mut fmt::Formatter, content: &str, sugg: &crate::Suggestion, labels: Vec<(usize, &crate::Label)>, padd: usize, far: &Files) -> fmt::Result {
  let space = " ".repeat(padd);

  let mut lines: Vec<LineRef<'_>> = Vec::new();
  let mut offset = 0;
  for slice in content.split('\n') {
    let clean = slice.trim_end_matches('\r');
    lines.push(LineRef {
      start_byte: offset,
      text: clean,
    });
    offset += slice.len() + 1;
  }

  // Collect intervals from all labels in this file
  let mut intervals: Vec<(u32, u32)> = labels
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
      writeln!(f, " {}{}", " ".repeat(padd-1), "...".bright_black())?;
    }

    let mut skip_until_line = 0u32;

    for line_no in b_start..=b_end {
      if line_no < skip_until_line {
        continue;
      }

      let line_idx = (line_no - 1) as usize;
      let line = match lines.get(line_idx) {
        Some(l) => l,
        None => continue,
      };

      // İlk label (orig_idx == 0) bu satırda mı başlıyor?
      let primary_label_opt = labels.iter().find(|(orig_idx, l)| {
        if *orig_idx == 0 {
          let hr = l.span.interval(far);
          hr.start[0].get() == line_no
        } else {
          false
        }
      });

      if let Some((_, primary_label)) = primary_label_opt {
        let hr = primary_label.span.interval(far);
        let start_line = hr.start[0].get();
        let end_line = hr.end[0].get();
        let rng = primary_label.span.range();

        if start_line == end_line {
          let line_byte_start = line.start_byte;
          let line_len = line.text.len();
          let rel_start = rng.start.saturating_sub(line_byte_start).min(line_len);
          let rel_end = rng.end.saturating_sub(line_byte_start).min(line_len);

          let prefix = safe_slice_up_to(line.text, rel_start);
          let suffix = safe_slice_from(line.text, rel_end);

          if !sugg.replacement.contains('\n') {
            let modified_line = format!("{}{}{}", prefix, sugg.replacement.yellow(), suffix);
            let line_str = start_line.to_string();
            let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));
            writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), modified_line)?;

            // İlk label `~` ile değişimi gösterir
            let mut underline = String::new();
            for ch in prefix.chars() {
              if ch == '\t' { underline.push('\t'); } else { underline.push(' '); }
            }
            let marker_count = if sugg.replacement.is_empty() { 1 } else { sugg.replacement.chars().count() };
            underline.extend(std::iter::repeat('~').take(marker_count));
            if let Some(msg) = primary_label.message.as_deref().filter(|m| !m.is_empty()) {
              writeln!(f, " {} {} {} {}", space, "|".bright_black(), underline.green().bold(), msg.green().bold())?;
            } else {
              writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
            }

            // Diğerleri ise `-` ile ek not girer
            for (orig_idx, sec_label) in &labels {
              if *orig_idx > 0 {
                let sec_rng = sec_label.span.range();
                if let Some(sec_underline) = build_underline(line.text, line.start_byte, sec_rng.start, sec_rng.end, false) {
                  let (colored_underline, colored_msg) = color_underline(&sec_underline, sec_label.message.as_deref(), false);
                  if colored_msg.is_empty() {
                    writeln!(f, " {} {} {}", space, "|".bright_black(), colored_underline)?;
                  } else {
                    writeln!(f, " {} {} {} {}", space, "|".bright_black(), colored_underline, colored_msg)?;
                  }
                }
              }
            }
          } else {
            // Multi-line replacement
            let rep_lines: Vec<&str> = sugg.replacement.split('\n').map(|s| s.trim_end_matches('\r')).collect();
            for (i, rep_line) in rep_lines.iter().enumerate() {
              let curr_line_no = start_line + i as u32;
              let line_str = curr_line_no.to_string();
              let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));

              if i == 0 {
                let mod_first = format!("{}{}", prefix, rep_line.yellow());
                writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), mod_first)?;

                let mut underline = String::new();
                for ch in prefix.chars() {
                  if ch == '\t' { underline.push('\t'); } else { underline.push(' '); }
                }
                underline.extend(std::iter::repeat('~').take(rep_line.chars().count()));
                writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
              } else if i == rep_lines.len() - 1 {
                let mod_last = format!("{}{}", rep_line.yellow(), suffix);
                writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), mod_last)?;

                let underline: String = std::iter::repeat('~').take(rep_line.chars().count()).collect();
                if let Some(msg) = primary_label.message.as_deref().filter(|m| !m.is_empty()) {
                  writeln!(f, " {} {} {} {}", space, "|".bright_black(), underline.green().bold(), msg.green().bold())?;
                } else {
                  writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
                }
              } else {
                writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), rep_line.yellow())?;

                let underline: String = std::iter::repeat('~').take(rep_line.chars().count()).collect();
                writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
              }
            }
          }
        } else {
          // Multi-line span
          skip_until_line = end_line + 1;

          let start_idx = (start_line - 1) as usize;
          let end_idx = (end_line - 1) as usize;
          if let (Some(first_line), Some(last_line)) = (lines.get(start_idx), lines.get(end_idx)) {
            let rel_start = rng.start.saturating_sub(first_line.start_byte).min(first_line.text.len());
            let rel_end = rng.end.saturating_sub(last_line.start_byte).min(last_line.text.len());

            let prefix = safe_slice_up_to(first_line.text, rel_start);
            let suffix = safe_slice_from(last_line.text, rel_end);

            let rep_lines: Vec<&str> = sugg.replacement.split('\n').map(|s| s.trim_end_matches('\r')).collect();
            if rep_lines.len() == 1 {
              let mod_line = format!("{}{}{}", prefix, sugg.replacement.yellow(), suffix);
              let line_str = start_line.to_string();
              let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));
              writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), mod_line)?;

              let mut underline = String::new();
              for ch in prefix.chars() {
                if ch == '\t' { underline.push('\t'); } else { underline.push(' '); }
              }
              let marker_count = if sugg.replacement.is_empty() { 1 } else { sugg.replacement.chars().count() };
              underline.extend(std::iter::repeat('~').take(marker_count));
              if let Some(msg) = primary_label.message.as_deref().filter(|m| !m.is_empty()) {
                writeln!(f, " {} {} {} {}", space, "|".bright_black(), underline.green().bold(), msg.green().bold())?;
              } else {
                writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
              }
            } else {
              for (i, rep_line) in rep_lines.iter().enumerate() {
                let curr_line_no = start_line + i as u32;
                let line_str = curr_line_no.to_string();
                let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));

                if i == 0 {
                  let mod_first = format!("{}{}", prefix, rep_line.yellow());
                  writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), mod_first)?;

                  let mut underline = String::new();
                  for ch in prefix.chars() {
                    if ch == '\t' { underline.push('\t'); } else { underline.push(' '); }
                  }
                  underline.extend(std::iter::repeat('~').take(rep_line.chars().count()));
                  writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
                } else if i == rep_lines.len() - 1 {
                  let mod_last = format!("{}{}", rep_line.yellow(), suffix);
                  writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), mod_last)?;

                  let underline: String = std::iter::repeat('~').take(rep_line.chars().count()).collect();
                  if let Some(msg) = primary_label.message.as_deref().filter(|m| !m.is_empty()) {
                    writeln!(f, " {} {} {} {}", space, "|".bright_black(), underline.green().bold(), msg.green().bold())?;
                  } else {
                    writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
                  }
                } else {
                  writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), rep_line.yellow())?;

                  let underline: String = std::iter::repeat('~').take(rep_line.chars().count()).collect();
                  writeln!(f, " {} {} {}", space, "|".bright_black(), underline.green().bold())?;
                }
              }
            }
          }
        }
      } else {
        // Primary label bu satırda değil, normal satır ve ikincil labeller
        let line_str = line_no.to_string();
        let left_pad = " ".repeat(padd.saturating_sub(line_str.len()));
        writeln!(f, " {}{} {} {}", left_pad, line_str, "|".bright_black(), line.text)?;

        // Diğerleri ise `-` ile ek not girer
        for (orig_idx, label) in &labels {
          if *orig_idx > 0 {
            let rng = label.span.range();
            if let Some(underline) = build_underline(line.text, line.start_byte, rng.start, rng.end, false) {
              let (colored_underline, colored_msg) = color_underline(&underline, label.message.as_deref(), false);
              if colored_msg.is_empty() {
                writeln!(f, " {} {} {}", space, "|".bright_black(), colored_underline)?;
              } else {
                writeln!(f, " {} {} {} {}", space, "|".bright_black(), colored_underline, colored_msg)?;
              }
            }
          }
        }
      }
    }
  }

  writeln!(f, " {} {}", space, "|".bright_black())?;
  Ok(())
}

fn safe_slice_up_to(s: &str, mut idx: usize) -> &str {
  idx = idx.min(s.len());
  while idx > 0 && !s.is_char_boundary(idx) {
    idx -= 1;
  }
  &s[..idx]
}

fn safe_slice_from(s: &str, mut idx: usize) -> &str {
  idx = idx.min(s.len());
  while idx < s.len() && !s.is_char_boundary(idx) {
    idx += 1;
  }
  &s[idx..]
}
