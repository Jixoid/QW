/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include <CLI/CLI.hpp>
#include "qw/codegen/codegen.hh"
#include "qw/sys/sys.hh"
#include "qw/control/context.hh"
#include "qw/control/scopemng.hh"
#include "qw/front/front.hh"
#include "qw/pretype.hh"
#include "qw/sema/sema.hh"
#include "qw/vfs/vfs_file.hh"
#include "qw/ds/ds.hh"
#include <fstream>
#include <filesystem>
#include <iostream>
#include <llvm/IR/Type.h>
#include <llvm/Linker/Linker.h>
#include <llvm/Transforms/Utils/Cloning.h>
#include <llvm/Support/CodeGen.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/MC/TargetRegistry.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/TargetParser/Host.h>
#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Object/ArchiveWriter.h>
#include <lld/Common/Driver.h>

LLD_HAS_DRIVER(elf)
LLD_HAS_DRIVER(mingw)
LLD_HAS_DRIVER(coff)
LLD_HAS_DRIVER(macho)
LLD_HAS_DRIVER(wasm)

using namespace qw;



fun main(int argc, char** argv) -> int
{
  llvm::InitializeNativeTarget();
  llvm::InitializeNativeTargetAsmPrinter();
  llvm::InitializeNativeTargetAsmParser();

  CLI::App app{"QW Compiler"};
  app.require_subcommand(1);

  bool fpic = false;
  std::string rtl_path = "";
  std::string dst = "native";
  std::string path = std::filesystem::current_path().string();
  std::string out_file;

  auto build_cmd = app.add_subcommand("build", "Builds the QW project/module");
  build_cmd->add_flag("-f,--fpic", fpic, "Generate position independent code");
  build_cmd->add_option("--rtl", rtl_path, "Path to RTL library");
  build_cmd->add_option("--dst", dst, "Target destination (native, bytecode, llir, object, archive, shared, executable)")->check(CLI::IsMember({"native", "bytecode", "llir", "object", "archive", "shared", "executable"}));
  build_cmd->add_option("path", path, "Path to the project directory");

  auto run_cmd = app.add_subcommand("run", "Runs the QW project/module using lli");
  run_cmd->add_option("path", path, "Path to the project directory");
  run_cmd->add_option("--rtl", rtl_path, "Path to RTL library");

  CLI11_PARSE(app, argc, argv);

  if (build_cmd->parsed() || run_cmd->parsed()) {
    if (run_cmd->parsed())
      dst = "executable";

    auto vfs_file = vfs::__file::get();

    setlocale(LC_ALL, "");
    bindtextdomain("main", "/home/alforce/Projeler/QW/qw/i18n");
    textdomain("main");

    // Project Path and Manifest
    std::string manifest_path = path + "/manifest.conf";
    
    std::ifstream manifest_file(manifest_path);
    if (!manifest_file.is_open()) {
      std::cerr << "Manifest file could not be opened: " << manifest_path << std::endl;
      return 1;
    }
    
    auto manifest = ds::value::makeFromStreamRaw(&manifest_file);
    std::string project_name = manifest.isStruct() && !manifest["name"].isUnDef() ? manifest["name"].w_string() : "unknown";

    std::string main_qw_path = path + "/src/main.qw";
    std::string lib_qw_path = path + "/src/lib.qw";
    std::string entry_file = std::filesystem::exists(main_qw_path) ? main_qw_path : (std::filesystem::exists(lib_qw_path) ? lib_qw_path : "");

    if (entry_file.empty()) {
      std::cerr << "No main.qw or lib.qw found in " << path << std::endl;
      return 1;
    }

    // Prepare
    auto ctx = context::make(ProgBits::Bit64);
    auto mod = ctx->make_module(project_name, entry_file);
    ctx->mark_parsed(entry_file);

    // qw system unit
    auto sys = qw::sys::build_sys_module(ctx.get());

    bool use_no_rtl = false;
    llvm::Triple TheTriple(llvm::sys::getDefaultTargetTriple());
    std::string target_rtl_src = rtl_path + "/" + TheTriple.getOSName().str() + "-" + TheTriple.getArchName().str() + "/src";
    std::string target_rtl_obj = rtl_path + "/" + TheTriple.getOSName().str() + "-" + TheTriple.getArchName().str() + "/obj";
    qw::module* rtl_master_mod = nullptr;
    // Front
    l_pass_0: {
      auto Sum = frontend(mod).process();

      if (Sum.sumerr()) {
        std::cerr << Sum;
        goto l_final;
      }
      
      // Check for ![use no rtl]
      for (const auto& attr : mod->nameSpace()->const_attrs()) {
        if (attr.name == "use" && attr.value == "no rtl") {
          use_no_rtl = true;
          break;
        }
      }

      // If use_no_rtl, remove heap, vmt from sys and skip rtl parsing
      if (use_no_rtl) {
        auto& decls = sys->as<decls::NameSpaceDecl>()->decls;
        decls.erase(
          std::remove_if(decls.begin(), decls.end(), [](decls::Decl* d) {
            return d->name() == "heap" || d->name() == "vmt";
          }),
          decls.end()
        );
      }

      // Load RTL files if specified and allowed
      if (!rtl_path.empty() && !use_no_rtl) {
        if (std::filesystem::exists(target_rtl_src) && std::filesystem::is_directory(target_rtl_src)) {
          for (const auto& entry : std::filesystem::recursive_directory_iterator(target_rtl_src)) {
            if (entry.is_regular_file() && entry.path().extension() == ".qw") {
              std::string rel_path = std::filesystem::relative(entry.path(), target_rtl_src).string();
              
              decls::Decl* curr_ns = sys;
              std::filesystem::path p(rel_path);
              std::vector<std::string> parts;
              for (auto it = p.begin(); it != p.end(); ++it) {
                if (it == --p.end())
                  parts.push_back(p.stem().string());
                else
                  parts.push_back(it->string());
              }
              
              std::string full_ns_name = "sys";
              for (const auto &part: parts) {
                full_ns_name += "::" + part;
                decls::Decl* next_ns = nullptr;
                for (auto decl: curr_ns->as<decls::NameSpaceDecl>()->decls)
                  if (decl->is<decls::NameSpaceDecl>() && decl->name() == part) {
                    next_ns = decl;
                    break;
                  }
                
                if (!next_ns) {
                  next_ns = decls::Decl::make_NameSpace(ctx.get(), curr_ns, part, word{});
                }
                curr_ns = next_ns;
              }
              
              auto rtl_mod = ctx->make_module("__qwrtl_" + full_ns_name, entry.path().string());
              frontend rtl_front(rtl_mod);
              if (auto E = rtl_front.read_File(curr_ns); !E.has_value()) {
                std::cerr << E.error();
              }
            }
          }
        }
      }
    }

    // Sema
    l_pass_1: {
      if (!rtl_path.empty() && !use_no_rtl) {
        std::string first_rtl_file = target_rtl_src;
        if (std::filesystem::exists(target_rtl_src)) {
          for (const auto& entry : std::filesystem::recursive_directory_iterator(target_rtl_src)) {
            if (entry.is_regular_file() && entry.path().extension() == ".qw") {
              first_rtl_file = entry.path().string();
              break;
            }
          }
        }
        rtl_master_mod = ctx->make_module("qwrtl", first_rtl_file);
        auto Sum2 = qw::Sema::pass_ns(rtl_master_mod, sys, {});
        if (Sum2.sumerr()) {
          std::cerr << Sum2;
          return 1;
        }
      }

      auto Sum = Sema::pass(mod, { scopemng::mangling_abi_qw(sys) });

      if (Sum.sumerr()) {
        std::cerr << Sum;
        goto l_final;
      }
    }

    // CodeGen
    l_pass_2: {
      CodeGen::pass(mod, { scopemng::mangling_abi_qw(sys) });

      if (rtl_master_mod) {
        qw::CodeGen::pass_ns(rtl_master_mod, sys, {});
        
        bool err = llvm::Linker::linkModules(*mod->llvm(), llvm::CloneModule(*rtl_master_mod->llvm()));
        if (err) {
          std::cerr << "RTL moduulunu ana module baglarken hata olustu." << std::endl;
        }
      }
    }

    // LLVM
    l_pass_3: {
      mod->llvm()->setTargetTriple(TheTriple);

      std::string Error;
      auto Target = llvm::TargetRegistry::lookupTarget("", TheTriple, Error); // Use the one taking Triple
      if (!Target) {
        Target = llvm::TargetRegistry::lookupTarget(TheTriple, Error);
        if (!Target) {
          std::cerr << "LLVM Target Error: " << Error << std::endl;
          goto l_final;
        }
      }

      auto CPU = "generic";
      auto Features = "";
      llvm::TargetOptions opt;
      auto RM = std::optional<llvm::Reloc::Model>();

      if (fpic) RM = llvm::Reloc::PIC_;


      std::unique_ptr<llvm::TargetMachine> TM(Target->createTargetMachine(TheTriple, CPU, Features, opt, RM));
      if (!TM) {
        std::cerr << "LLVM TargetMachine Error" << std::endl;
        goto l_final;
      }

      mod->llvm()->setDataLayout(TM->createDataLayout());

      std::string ext_o = ".o";
      std::string ext_a = ".a";
      std::string ext_so = ".so";
      std::string ext_exe = "";
      
      if (TheTriple.isOSWindows()) {
        ext_o = ".obj";
        ext_a = ".lib";
        ext_so = ".dll";
        ext_exe = ".exe";
      }
      ef (TheTriple.isMacOSX()) {
        ext_so = ".dylib";
      }

      std::string base_file = path + "/build/out";
      
      if (dst == "bytecode")   out_file = base_file + ".bc";
      ef (dst == "llir")       out_file = base_file + ".ll";
      ef (dst == "object")     out_file = base_file + ext_o;
      ef (dst == "native")     out_file = base_file + ext_o;
      ef (dst == "archive")    out_file = base_file + ext_a;
      ef (dst == "shared")     out_file = base_file + ext_so;
      ef (dst == "executable") out_file = base_file + ext_exe;

      std::filesystem::create_directories(path + "/build");
      for (const auto& entry : std::filesystem::directory_iterator(path + "/build"))
        if (entry.path().filename().string().starts_with("out"))
          std::filesystem::remove(entry.path());
      

      std::error_code EC;
      
      if (dst == "bytecode") {
        llvm::raw_fd_ostream dest(out_file, EC, llvm::sys::fs::OF_None);
        if (EC) {
          std::cerr << "Could not open file: " << EC.message() << std::endl;
          goto l_final;
        }
        llvm::WriteBitcodeToFile(*mod->llvm(), dest);
        dest.flush();
        dest.close();
      }
      ef (dst == "llir") {
        mod->write(out_file);
      }
      else {
        std::string obj_file = (dst == "object" || dst == "native") ? out_file : (base_file + ext_o);
        llvm::raw_fd_ostream dest(obj_file, EC, llvm::sys::fs::OF_None);
        if (EC) {
          std::cerr << "Could not open file: " << EC.message() << std::endl;
          goto l_final;
        }

        llvm::legacy::PassManager pass;
        if (TM->addPassesToEmitFile(pass, dest, nullptr, llvm::CodeGenFileType::ObjectFile)) {
          std::cerr << "TargetMachine can't emit a file of this type" << std::endl;
          goto l_final;
        }

        pass.run(*mod->llvm());
        dest.flush();
        dest.close();

        if (dst == "archive") {
          auto newMemberOrErr = llvm::NewArchiveMember::getFile(obj_file, true);
          if (!newMemberOrErr) {
            std::cerr << "Could not read object file for archive" << std::endl;
          }
          else {
            std::vector<llvm::NewArchiveMember> members;
            members.push_back(std::move(*newMemberOrErr));
            auto Kind = TheTriple.isOSWindows() ? llvm::object::Archive::K_COFF : llvm::object::Archive::K_GNU;
            llvm::Error err = llvm::writeArchive(out_file, members, llvm::SymtabWritingMode::NormalSymtab, Kind, true, false, nullptr);
            if (err) {
              std::cerr << "Failed to create archive" << std::endl;
            }
          }
        }
        ef (dst == "shared" || dst == "executable") {
          std::vector<const char*> args;
          args.push_back("ld.lld");
          if (dst == "shared") args.push_back("-shared");
          
          std::string cqrt_o;
          if (!use_no_rtl && !rtl_path.empty() && dst == "executable") {
            args.push_back("-dynamic-linker");
            args.push_back("/lib64/ld-linux-x86-64.so.2");
            
            cqrt_o = target_rtl_obj + "/cqrt.o";
            if (std::filesystem::exists(cqrt_o)) {
              args.push_back(cqrt_o.c_str());
            } else {
              std::cerr << "Warning: " << cqrt_o << " not found!" << std::endl;
            }
          }

          args.push_back("-o");
          args.push_back(out_file.c_str());
          args.push_back(obj_file.c_str());

          if (!use_no_rtl && !rtl_path.empty() && dst == "executable") {
            args.push_back("-L/lib/x86_64-linux-gnu");
            args.push_back("-L/usr/lib/x86_64-linux-gnu");
            args.push_back("-L/lib64");
            args.push_back("-L/usr/lib64");
            args.push_back("-lc");
          }
          
          bool linkSuccess = false;
          if (TheTriple.isOSWindows())
            linkSuccess = lld::coff::link(args, llvm::outs(), llvm::errs(), false, false);
          ef (TheTriple.isMacOSX())
            linkSuccess = lld::macho::link(args, llvm::outs(), llvm::errs(), false, false);
          else
            linkSuccess = lld::elf::link(args, llvm::outs(), llvm::errs(), false, false);
          
          if (!linkSuccess)
            std::cerr << "LLD Linker failed." << std::endl;
        }
      }
    }

    // Final
    l_final:
    if (run_cmd->parsed()) {
      std::string run_cmd_str = out_file;
      int ret = system(run_cmd_str.c_str());
      if (ret != 0)
        std::cerr << "execution failed with code: " << ret << std::endl;
    }
  }

  return 0;
}
