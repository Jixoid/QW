use std::{panic, process::exit};

use backtrace::Backtrace;
use owo_colors::OwoColorize;


pub fn ice_bt() -> Backtrace { Backtrace::new() }

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = match option_env!("GIT_HASH") {
  Some(hash) => hash,
  None => "dev",
};


pub struct ICE;

impl ICE {

  pub fn new(
    bt: Backtrace,
    msg: &str,
  ) -> ! {
    eprintln!("{}{} {}{} {msg}", "internal compiler error".yellow().bold(), ":".bright_black(), "qw panicked!".red().bold(), ":".bright_black());

    eprintln!();
    eprintln!("{}", "qw has entered a state where it cannot continue due to an internal error.".white().bold());
    eprintln!("{}", "it will exit after generating a backtrace dump.".white().bold());
    eprintln!("{}", "please report this to us at: <https://github.com/jixoid/qw/issues>".white().bold());

    if !cfg!(debug_assertions) {
      eprintln!();
      eprintln!("{}", "if the output isn't detailed enough, you can try again using the qw.debug tool.".yellow().bold());
    }
    
    eprintln!();
    eprintln!("{}", "if you want to investigate further, you can debug the error using gdb.".green().bold());

    eprintln!();
    eprintln!("[INFO]");
    eprintln!("os: {}", std::env::consts::OS);
    eprintln!("arch: {}", std::env::consts::ARCH);
    eprintln!("vers: {VERSION}");
    eprintln!("git_hash: {GIT_HASH}");

    eprintln!();
    eprintln!("[BACKTRACE]");
    //eprintln!("{bt:?}");

    for (i, frame) in bt.frames().iter().enumerate() {
      for symbol in frame.symbols() {
        let file_path = symbol.filename();

        let mut ness = true;
        if let Some(path) = file_path {
          let path_str = path.to_string_lossy();
          if path_str.contains("/rustc/")
            || path_str.contains("/library/std/")
            || path_str.contains("/library/core/")
            || path_str.contains("/library/alloc/")
            || path_str.contains("qwc_diagnostic/src/ice.rs") // ICE yakalayıcının kendi satırları
          {
            ness = false;
          }
        }

        let str = {
          let func_name = symbol
            .name()
            .map(|n| format!("{n}"))
            .unwrap_or_else(|| "<unknown>".to_string());
          
          let path_display = file_path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "??".to_string());
          
          let line = symbol.lineno().unwrap_or(0);
          let col = symbol.colno().unwrap_or(0);
          
          if col > 0 {
            format!(" {i:>3}: {func_name}\n       at {}:{}:{}", path_display, line, col)
          } else if line > 0 {
            format!(" {i:>3}: {func_name}\n       at {}:{}", path_display, line)
          } else {
            format!(" {i:>3}: {func_name}\n       at {}", path_display)
          }
        };

        if ness { eprintln!("{str}") } else { eprintln!("{}", str.bright_black()) }
      }
    }

    exit(1)
  }


  pub fn register() {
    panic::set_hook(Box::new(|phi| {
      ICE::new(Backtrace::new(),
        match phi.payload_as_str() {
          Some(v) => v,
          None => "<unknown>",
        }
      )
    }));
  }

}
