#include "qw/lsp/server.hh"
#include "qw/vfs/vfs_file.hh"
#include "qw/front/front.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/control/context.hh"
#include <iostream>
#include <string>
#include <algorithm>
#include <nlohmann/json.hpp>

using json = nlohmann::json;



namespace qw::lsp
{

  fun strip_ansi(const std::string& input) -> std::string {
    std::string output;
    bool in_escape = false;
    for (char c : input) {
      if (c == '\x1B') {
        in_escape = true;
      }
      ef (in_escape) {
        if (c == 'm')
          in_escape = false;
      }
      else {
        output += c;
      }
    }
    return output;
  }

  fun compile_and_report(const std::string &uri, const std::string &fpath) -> void {
    json diagnostics = json::array();

    try {
      auto vfs_file = vfs::__file::get(); 
      auto ctx = qw::context::make(qw::ProgBits::Bit64);
      auto mod = ctx->make_module("lsp_mod", fpath);
      ctx->mark_parsed(fpath);

      qw::frontend front(mod);
      auto Sum = front.process(); 

      for (auto& msg : Sum.messages()) {
        auto [pos1, pos2] = msg.pos().interval();
        int severity = 1; 
        switch (msg.type()) {
          case qw::diagnostic::MsgType::Fatal:
          case qw::diagnostic::MsgType::Error: severity = 1; break;
          case qw::diagnostic::MsgType::Warning: severity = 2; break;
          case qw::diagnostic::MsgType::Hint: severity = 4; break;
          case qw::diagnostic::MsgType::Note: severity = 3; break;
        }

        std::string formatted_msg = qw::diagnostic::format_from_vector(msg.msg(), msg.params());
        formatted_msg = strip_ansi(formatted_msg);

        diagnostics.push_back({
          {"range", {
            {"start", {{"line", std::max(0, (int)pos1.y - 1)}, {"character", std::max(0, (int)pos1.x - 1)}}},
            {"end", {{"line", std::max(0, (int)pos2.y - 1)}, {"character", std::max(0, (int)pos2.x - 1)}}}
          }},
          {"severity", severity},
          {"message", formatted_msg}
        });
      }
    } catch (const std::exception &e) {
      std::string err_msg = e.what();
      err_msg = strip_ansi(err_msg);
      diagnostics.push_back({
        {"range", {
          {"start", {{"line", 0}, {"character", 0}}},
          {"end", {{"line", 0}, {"character", 0}}}
        }},
        {"severity", 1},
        {"message", "Fatal compiler error: " + err_msg}
      });
    }

    json notification = {
      {"jsonrpc", "2.0"},
      {"method", "textDocument/publishDiagnostics"},
      {"params", {
        {"uri", uri},
        {"diagnostics", diagnostics}
      }}
    };

    std::string res_str = notification.dump();
    std::cout << "Content-Length: " << res_str.size() << "\r\n\r\n" << res_str << std::flush;
  }

  fun generate_semantic_tokens(const std::string &uri, const std::string &fpath) -> json {
    auto vfs_file = vfs::__file::get(); 
    auto ctx = qw::context::make(qw::ProgBits::Bit64);
    auto mod = ctx->make_module("lsp_mod", fpath);

    qw::lexer lexer(mod);
    std::vector<int> data;
    
    int prev_line = 0;
    int prev_char = 0;

    while (true) {
      auto w = lexer();
      if (!w) break;

      auto view = w.view();
      if (view.empty()) continue;

      auto [pos1, pos2] = w.interval();
      int line = std::max(0, (int)pos1.y - 1);
      int character = std::max(0, (int)pos1.x - 1);
      int length = view.length();

      int tokenType = -1;
      auto kind = lexer.kind(view[0]);
      
      if (kind & qw::CharKind::String) tokenType = 1;
      ef (kind & qw::CharKind::Numeral) tokenType = 2;
      ef (kind & qw::CharKind::Symbol) tokenType = 3;
      ef (kind & qw::CharKind::Word) {
        std::string v(view);
        if (v == "fun" || v == "struct" || v == "var" || v == "alias" || v == "type" || 
            v == "iface" || v == "enum" || v == "set" || v == "mod" || v == "if" || 
            v == "while" || v == "ef" || v == "pub" || v == "priv" || v == "prot" || 
            v == "crate" || v == "group" || v == "init" || v == "fini" || v == "return") {
          tokenType = 0; // keyword
        } else {
          tokenType = 8; // variable
        }
      }

      if (tokenType != -1) {
        int deltaLine = line - prev_line;
        int deltaStart = (deltaLine == 0) ? (character - prev_char) : character;
        
        data.push_back(deltaLine);
        data.push_back(deltaStart);
        data.push_back(length);
        data.push_back(tokenType);
        data.push_back(0); // modifier

        prev_line = line;
        prev_char = character;
      }
    }
    return data;
  }

  fun extract_fpath(const std::string &uri) -> std::string {
    if (uri.starts_with("file://")) {
      return uri.substr(7);
    }
    return uri;
  }

  fun run_server() -> int {
    qw::diagnostic::is_lsp_mode = true;
    std::cerr << "QW Language Server started.\n";

    while (std::cin.good()) {
      std::string line;
      std::getline(std::cin, line);
      
      // Ignore empty lines
      if (line.empty() || line == "\r") continue;

      // Check for Content-Length header
      if (line.starts_with("Content-Length: ")) {
        size_t content_length = std::stoull(line.substr(16));
        
        // Read headers until the empty line separating headers from content
        while (std::getline(std::cin, line)) {
          if (line.empty() || line == "\r") break;
        }
        
        // Read the content
        std::string content(content_length, ' ');
        std::cin.read(content.data(), content_length);

        try {
          auto req = json::parse(content);
          
          // Very basic handling of the 'initialize' request
          if (req.contains("method") && req["method"] == "initialize") {
            json response = {
              {"jsonrpc", "2.0"},
              {"id", req["id"]},
              {"result", {
                {"capabilities", {
                  {"textDocumentSync", 1}, // 1 = Full sync
                  {"hoverProvider", false},
                  {"definitionProvider", false},
                  {"semanticTokensProvider", {
                    {"legend", {
                      {"tokenTypes", {"keyword", "string", "number", "operator", "comment", "type", "class", "function", "variable"}},
                      {"tokenModifiers", json::array()}
                    }},
                    {"full", true}
                  }}
                }},
                {"serverInfo", {
                  {"name", "qw-lsp"},
                  {"version", "0.1.0"}
                }}
              }}
            };
            
            std::string res_str = response.dump();
            std::cout << "Content-Length: " << res_str.size() << "\r\n\r\n" << res_str << std::flush;
          }
          ef (req.contains("method") && req["method"] == "textDocument/didOpen") {
            auto uri = req["params"]["textDocument"]["uri"].get<std::string>();
            auto text = req["params"]["textDocument"]["text"].get<std::string>();
            auto fpath = extract_fpath(uri);
            vfs::__file::set_override(fpath, text);
            compile_and_report(uri, fpath);
          }
          ef (req.contains("method") && req["method"] == "textDocument/didChange") {
            auto uri = req["params"]["textDocument"]["uri"].get<std::string>();
            auto text = req["params"]["contentChanges"][0]["text"].get<std::string>();
            auto fpath = extract_fpath(uri);
            vfs::__file::set_override(fpath, text);
            compile_and_report(uri, fpath);
          }
          ef (req.contains("method") && req["method"] == "textDocument/didClose") {
            auto uri = req["params"]["textDocument"]["uri"].get<std::string>();
            auto fpath = extract_fpath(uri);
            vfs::__file::remove_override(fpath);
          }
          ef (req.contains("method") && req["method"] == "textDocument/semanticTokens/full") {
            auto uri = req["params"]["textDocument"]["uri"].get<std::string>();
            auto fpath = extract_fpath(uri);
            json data = generate_semantic_tokens(uri, fpath);
            
            json response = {
              {"jsonrpc", "2.0"},
              {"id", req["id"]},
              {"result", {
                {"data", data}
              }}
            };
            std::string res_str = response.dump();
            std::cout << "Content-Length: " << res_str.size() << "\r\n\r\n" << res_str << std::flush;
          }
          ef (req.contains("method") && req["method"] == "shutdown") {
            json response = {
              {"jsonrpc", "2.0"},
              {"id", req["id"]},
              {"result", nullptr}
            };
            std::string res_str = response.dump();
            std::cout << "Content-Length: " << res_str.size() << "\r\n\r\n" << res_str << std::flush;
          }
          ef (req.contains("method") && req["method"] == "exit") {
            break;
          }
        } catch (const std::exception &e) {
          std::cerr << "JSON parse error: " << e.what() << "\n";
        }
      }
    }

    std::cerr << "QW Language Server stopped.\n";
    return 0;
  }

}
