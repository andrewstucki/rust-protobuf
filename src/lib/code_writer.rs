use std::io::Write;

// TODO: should not use wire_format here
use wire_format;

pub struct CodeWriter<'a> {
    writer: &'a mut (Write + 'a),
    indent: String,
}

impl<'a> CodeWriter<'a> {
    pub fn new(writer: &'a mut Write) -> CodeWriter<'a> {
        CodeWriter {
            writer: writer,
            indent: "".to_string(),
        }
    }

    pub fn write_line<S : AsRef<str>>(&mut self, line: S) {
        (if line.as_ref().is_empty() {
            self.writer.write_all("\n".as_bytes())
        } else {
            let s: String = [self.indent.as_ref(), line.as_ref(), "\n"].concat();
            self.writer.write_all(s.as_bytes())
        }).unwrap();
    }

    pub fn write_generated(&mut self) {
        self.write_line("// This file is generated. Do not edit");

        // https://secure.phabricator.com/T784
        self.write_line("// @generated");

        self.write_line("");
        self.comment("https://github.com/Manishearth/rust-clippy/issues/702");
        self.write_line("#![allow(unknown_lints)]");
        self.write_line("#![allow(clippy)]");
        self.write_line("");
        self.write_line("#![cfg_attr(rustfmt, rustfmt_skip)]");
        self.write_line("");
        self.write_line("#![allow(box_pointers)]");
        self.write_line("#![allow(dead_code)]");
        self.write_line("#![allow(missing_docs)]");
        self.write_line("#![allow(non_camel_case_types)]");
        self.write_line("#![allow(non_snake_case)]");
        self.write_line("#![allow(non_upper_case_globals)]");
        self.write_line("#![allow(trivial_casts)]");
        self.write_line("#![allow(unsafe_code)]");
        self.write_line("#![allow(unused_imports)]");
        self.write_line("#![allow(unused_results)]");
    }

    pub fn todo(&mut self, message: &str) {
        self.write_line(format!("panic!(\"TODO: {}\");", message));
    }

    pub fn unimplemented(&mut self) {
        self.write_line(format!("unimplemented!();"));
    }

    pub fn indented<F>(&mut self, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("{}    ", self.indent),
        });
    }

    #[allow(dead_code)]
    pub fn commented<F>(&mut self, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("// {}", self.indent),
        });
    }

    pub fn pub_const(&mut self, name: &str, field_type: &str, init: &str) {
        self.write_line(&format!("pub const {}: {} = {};", name, field_type, init));
    }

    pub fn lazy_static(&mut self, name: &str, ty: &str) {
        self.stmt_block(&format!("static mut {}: ::protobuf::lazy::Lazy<{}> = ::protobuf::lazy::Lazy", name, ty), |w| {
            w.field_entry("lock", "::protobuf::lazy::ONCE_INIT");
            w.field_entry("ptr", &format!("0 as *const {}", ty));
        });
    }

    pub fn lazy_static_decl_get<F>(&mut self, name: &str, ty: &str, init: F)
        where F : Fn(&mut CodeWriter)
    {
        self.lazy_static(name, ty);
        self.unsafe_expr(|w| {
            w.write_line(&format!("{}.get(|| {{", name));
            w.indented(|w| init(w));
            w.write_line(&format!("}})"));
        });
    }

    pub fn lazy_static_decl_get_simple(&mut self, name: &str, ty: &str, init: &str)
    {
        self.lazy_static(name, ty);
        self.unsafe_expr(|w| {
            w.write_line(&format!("{}.get({})", name, init));
        });
    }

    pub fn block<F>(&mut self, first_line: &str, last_line: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.write_line(first_line);
        self.indented(cb);
        self.write_line(last_line);
    }

    pub fn expr_block<F>(&mut self, prefix: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.block(&format!("{} {{", prefix), "}", cb);
    }

    pub fn stmt_block<S : AsRef<str>, F>(&mut self, prefix: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.block(&format!("{} {{", prefix.as_ref()), "};", cb);
    }

    pub fn unsafe_expr<F>(&mut self, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block("unsafe", cb);
    }

    pub fn impl_self_block<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("impl {}", name.as_ref()), cb);
    }

    pub fn impl_for_block<S1 : AsRef<str>, S2 : AsRef<str>, F>(&mut self, tr: S1, ty: S2, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("impl {} for {}", tr.as_ref(), ty.as_ref()), cb);
    }

    pub fn unsafe_impl(&mut self, what: &str, for_what: &str) {
        self.write_line(&format!("unsafe impl {} for {} {{}}", what, for_what));
    }

    pub fn pub_struct<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("pub struct {}", name.as_ref()), cb);
    }

    pub fn def_struct<S : AsRef<str>, F>(&mut self, name: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("struct {}", name.as_ref()), cb);
    }

    pub fn pub_enum<F>(&mut self, name: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("pub enum {}", name), cb);
    }

    pub fn pub_trait<F>(&mut self, name: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("pub trait {}", name), cb);
    }

    pub fn field_entry(&mut self, name: &str, value: &str) {
        self.write_line(&format!("{}: {},", name, value));
    }

    pub fn field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("{}: {},", name, field_type));
    }

    pub fn pub_field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("pub {}: {},", name, field_type));
    }

    pub fn derive(&mut self, derive: &[&str]) {
        let v: Vec<String> = derive.iter().map(|&s| s.to_string()).collect();
        self.write_line(&format!("#[derive({})]", v.join(",")));
    }

    pub fn allow(&mut self, what: &[&str]) {
        let v: Vec<String> = what.iter().map(|&s| s.to_string()).collect();
        self.write_line(&format!("#[allow({})]", v.join(",")));
    }

    pub fn comment(&mut self, comment: &str) {
        if comment.is_empty() {
            self.write_line("//");
        } else {
            self.write_line(&format!("// {}", comment));
        }
    }

    pub fn fn_def(&mut self, sig: &str) {
        self.write_line(&format!("fn {};", sig));
    }

    pub fn fn_block<F>(&mut self, public: bool, sig: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        if public {
            self.expr_block(&format!("pub fn {}", sig), cb);
        } else {
            self.expr_block(&format!("fn {}", sig), cb);
        }
    }

    pub fn pub_fn<F>(&mut self, sig: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.fn_block(true, sig, cb);
    }

    pub fn def_fn<F>(&mut self, sig: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.fn_block(false, sig, cb);
    }

    pub fn def_mod<F>(&mut self, name: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("mod {}", name), cb)
    }

    pub fn pub_mod<F>(&mut self, name: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("pub mod {}", name), cb)
    }

    pub fn while_block<S : AsRef<str>, F>(&mut self, cond: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("while {}", cond.as_ref()), cb);
    }

    // if ... { ... }
    pub fn if_stmt<S : AsRef<str>, F>(&mut self, cond: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("if {}", cond.as_ref()), cb);
    }

    // if ... {} else { ... }
    pub fn if_else_stmt<S : AsRef<str>, F>(&mut self, cond: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.write_line(&format!("if {} {{", cond.as_ref()));
        self.write_line("} else {");
        self.indented(cb);
        self.write_line("}");
    }

    // if let ... = ... { ... }
    pub fn if_let_stmt<F>(&mut self, decl: &str, expr: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.if_stmt(&format!("let {} = {}", decl, expr), cb);
    }

    // if let ... = ... { } else { ... }
    pub fn if_let_else_stmt<F>(&mut self, decl: &str, expr: &str, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.if_else_stmt(&format!("let {} = {}", decl, expr), cb);
    }

    pub fn for_stmt<S1 : AsRef<str>, S2 : AsRef<str>, F>(&mut self, over: S1, varn: S2, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.stmt_block(&format!("for {} in {}", varn.as_ref(), over.as_ref()), cb)
    }

    pub fn match_block<S : AsRef<str>, F>(&mut self, value: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.stmt_block(&format!("match {}", value.as_ref()), cb);
    }

    pub fn match_expr<S : AsRef<str>, F>(&mut self, value: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.expr_block(&format!("match {}", value.as_ref()), cb);
    }

    pub fn case_block<S : AsRef<str>, F>(&mut self, cond: S, cb: F)
        where F : Fn(&mut CodeWriter)
    {
        self.block(&format!("{} => {{", cond.as_ref()), "},", cb);
    }

    pub fn case_expr<S1 : AsRef<str>, S2 : AsRef<str>>(&mut self, cond: S1, body: S2) {
        self.write_line(&format!("{} => {},", cond.as_ref(), body.as_ref()));
    }

    pub fn error_unexpected_wire_type(&mut self, wire_type: &str) {
        self.write_line(&format!(
                "return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type({}));",
                wire_type));
    }

    pub fn assert_wire_type(&mut self, wire_type: wire_format::WireType) {
        self.if_stmt(&format!("wire_type != ::protobuf::wire_format::{:?}", wire_type), |w| {
            w.error_unexpected_wire_type("wire_type");
        });
    }
}
