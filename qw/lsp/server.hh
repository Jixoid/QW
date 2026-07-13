#pragma once

#include "qw/basis.hh"


namespace qw::lsp {

  /// @brief Starts the QW Language Server loop, reading from stdin and writing to stdout.
  /// @return Exit code for the program.
  fun run_server() -> int;

}
