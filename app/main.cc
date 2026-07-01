/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include <CLI/CLI.hpp>
#include "qw/codegen/codegen.hh"
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

  auto build_cmd = app.add_subcommand("build", "Builds the QW project/module");
  build_cmd->add_flag("-f,--fpic", fpic, "Generate position independent code");
  build_cmd->add_option("--rtl", rtl_path, "Path to RTL library");
  build_cmd->add_option("--dst", dst, "Target destination (native, bytecode, object, archive, shared, executable)")->check(CLI::IsMember({"native", "bytecode", "object", "archive", "shared", "executable"}));
  build_cmd->add_option("path", path, "Path to the project directory");

  CLI11_PARSE(app, argc, argv);

  if (build_cmd->parsed()) {
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

    // qw system unit
    auto sys = decls::Decl::make_NameSpace(ctx.get(), nil, "sys", word{});

    l_sys: {
      #define NewSys(Name, Type) { \
        auto tdecl = decls::Decl::make_Type(ctx.get(), sys, #Name, word{}, Type); \
        ctx->gst().add_ident(scopemng::mangling_abi_qw(tdecl), tdecl); \
      }

      // Types
      NewSys(u8, ctx->intU8_t()) NewSys(u16, ctx->intU16_t()) NewSys(u32, ctx->intU32_t()) NewSys(u64, ctx->intU64_t()) NewSys(u128, ctx->intU128_t())
      NewSys(i8, ctx->intS8_t()) NewSys(i16, ctx->intS16_t()) NewSys(i32, ctx->intS32_t()) NewSys(i64, ctx->intS64_t()) NewSys(i128, ctx->intS128_t())
      NewSys(f16, ctx->flo16_t()) NewSys(f32, ctx->flo32_t()) NewSys(f64, ctx->flo64_t()) NewSys(f128, ctx->flo128_t())
      NewSys(char, ctx->char_t())
      NewSys(bool, ctx->bool_t())
      NewSys(void, ctx->void_t())
      NewSys(ptr, ctx->ptr_t())

      #undef NewSys
      
      // RTL Bindings
      ctx->sys_api.sys_ns = sys;
      ctx->sys_api.heap_ns = decls::Decl::make_NameSpace(ctx.get(), sys, "heap", word{});
      sys->as<decls::NameSpaceDecl>()->decls.push_back(ctx->sys_api.heap_ns);
      
      // sys::heap::alloc(align: u64, size: u64) -> ptr
      ctx->sys_api.heap_alloc = decls::Decl::make_Func(ctx.get(), ctx->sys_api.heap_ns, "alloc", word{}, types::Type::make_Func(ctx.get(), {{"align", ctx->intU64_t()}, {"size", ctx->intU64_t()}}, ctx->ptr_t()), Visibility::Public);
      ctx->sys_api.heap_ns->as<decls::NameSpaceDecl>()->decls.push_back(ctx->sys_api.heap_alloc);
      ctx->gst().add_ident("qwrtl_heap_alloc", ctx->sys_api.heap_alloc);

      // sys::heap::dispose(p: ptr, align: u64, size: u64) -> void
      ctx->sys_api.heap_dispose = decls::Decl::make_Func(ctx.get(), ctx->sys_api.heap_ns, "dispose", word{}, types::Type::make_Func(ctx.get(), {{"p", ctx->ptr_t()}, {"align", ctx->intU64_t()}, {"size", ctx->intU64_t()}}, ctx->void_t()), Visibility::Public);
      ctx->sys_api.heap_ns->as<decls::NameSpaceDecl>()->decls.push_back(ctx->sys_api.heap_dispose);
      ctx->gst().add_ident("qwrtl_heap_dispose", ctx->sys_api.heap_dispose);

      // sys::heap::realloc(p: ptr, align: u64, old_size: u64, new_size: u64) -> ptr
      ctx->sys_api.heap_realloc = decls::Decl::make_Func(ctx.get(), ctx->sys_api.heap_ns, "realloc", word{}, types::Type::make_Func(ctx.get(), {{"p", ctx->ptr_t()}, {"align", ctx->intU64_t()}, {"old_size", ctx->intU64_t()}, {"new_size", ctx->intU64_t()}}, ctx->ptr_t()), Visibility::Public);
      ctx->sys_api.heap_ns->as<decls::NameSpaceDecl>()->decls.push_back(ctx->sys_api.heap_realloc);
      ctx->gst().add_ident("qwrtl_heap_realloc", ctx->sys_api.heap_realloc);
    }

    // Front
    std::cerr << color::RED << "# Front" << color::RESET << std::endl;
    l_pass_0: {
      auto Sum = frontend(mod).process();

      if (Sum.sumerr()) {
        std::cerr << Sum;
        goto l_final;
      }
    }

    // Sema
    std::cerr << color::RED << "# Sema" << color::RESET << std::endl;
    l_pass_1: {
      auto Sum = Sema::pass(mod, { scopemng::mangling_abi_qw(sys) });

      if (Sum.sumerr()) {
        std::cerr << Sum;
        goto l_final;
      }
    }

    // CodeGen
    std::cerr << color::RED << "# CodeGen" << color::RESET << std::endl;
    l_pass_2: {
      CodeGen::pass(mod, { scopemng::mangling_abi_qw(sys) });
    }

    // LLVM
    std::cerr << color::RED << "# LLVM" << color::RESET << std::endl;
    
    l_pass_3: {
      llvm::Triple TheTriple(llvm::sys::getDefaultTargetTriple());
      mod->llvm()->setTargetTriple(TheTriple);

      std::string Error;
      auto Target = llvm::TargetRegistry::lookupTarget("", TheTriple, Error); // Use the one taking Triple
      if (!Target) {
        Target = llvm::TargetRegistry::lookupTarget(TheTriple.str(), Error); // Fallback
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
      std::string out_file;
      
      if (dst == "bytecode")   out_file = base_file + ".bc";
      ef (dst == "object")     out_file = base_file + ext_o;
      ef (dst == "native")     out_file = base_file + ext_o;
      ef (dst == "archive")    out_file = base_file + ext_a;
      ef (dst == "shared")     out_file = base_file + ext_so;
      ef (dst == "executable") out_file = base_file + ext_exe;

      std::filesystem::create_directories(path + "/build");
      for (const auto& entry : std::filesystem::directory_iterator(path + "/build")) {
        if (entry.path().filename().string().starts_with("out")) {
          std::filesystem::remove(entry.path());
        }
      }

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
          args.push_back("-o");
          args.push_back(out_file.c_str());
          args.push_back(obj_file.c_str());
          
          bool linkSuccess = false;
          if (TheTriple.isOSWindows()) {
            linkSuccess = lld::coff::link(args, llvm::outs(), llvm::errs(), false, false);
          }
          ef (TheTriple.isMacOSX()) {
            linkSuccess = lld::macho::link(args, llvm::outs(), llvm::errs(), false, false);
          }
          else {
            linkSuccess = lld::elf::link(args, llvm::outs(), llvm::errs(), false, false);
          }
          
          if (!linkSuccess) {
            std::cerr << "LLD Linker failed." << std::endl;
          }
        }
      }
    }

    // Final
    l_final:
  }

  return 0;
}
