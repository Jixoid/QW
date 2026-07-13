#include "qw/lsp/server.hh"
#include <iostream>
#include <string>
#include <nlohmann/json.hpp>

using json = nlohmann::json;



namespace qw::lsp {

  fun run_server() -> int {
    std::cerr << "QW Language Server started.\n";

    while (std::cin.good()) {
      std::string line;
      std::getline(std::cin, line);
      
      // Ignore empty lines
      if (line.empty() || line == "\r") continue;

      // Check for Content-Length header
      if (line.starts_with("Content-Length: ")) {
        size_t content_length = std::stoull(line.substr(16));
        
        // Read the empty line separating headers from content
        std::getline(std::cin, line);
        
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
                  {"definitionProvider", false}
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
          else if (req.contains("method") && req["method"] == "shutdown") {
            json response = {
              {"jsonrpc", "2.0"},
              {"id", req["id"]},
              {"result", nullptr}
            };
            std::string res_str = response.dump();
            std::cout << "Content-Length: " << res_str.size() << "\r\n\r\n" << res_str << std::flush;
          }
          else if (req.contains("method") && req["method"] == "exit") {
            break;
          }
        } catch (const std::exception& e) {
          std::cerr << "JSON parse error: " << e.what() << "\n";
        }
      }
    }

    std::cerr << "QW Language Server stopped.\n";
    return 0;
  }

}
