use std::fmt::Write;
use crate::ast::{Expr, Program, Stmt, TopDecl};

pub fn emit(program: &Program) -> String {
    let mut out = String::new();
    for decl in &program.decls {
        emit_top_decl(&mut out, decl);
    }
    out.push('\n');
    out.push_str("void main() {\n");
    for stmt in &program.stmts {
        out.push_str("    ");
        emit_stmt(&mut out, stmt);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn emit_top_decl(out: &mut String, decl: &TopDecl) {
    match decl {
        TopDecl::Precision {qual, ty} => writeln!(out, "precision {qual} {ty};").unwrap(),
        TopDecl::Uniform {ty, name} => writeln!(out, "uniform {} {name};", gator_to_glsl(ty)).unwrap(),
        TopDecl::Varying {ty, name} => writeln!(out, "varying {} {name};", gator_to_glsl(ty)).unwrap()
    }
}

fn emit_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::Assign {name, expr} => {
            write!(out, "{name} = ").unwrap();
            emit_expr(out, expr);
            out.push(';');
        }
        Stmt::Decl {ty, name, expr} => {
            write!(out, "{} {name} = ", gator_to_glsl(ty)).unwrap();
            emit_expr(out, expr);
            out.push(';');
        }
        Stmt::SwizzleAssign {name, field, expr} => {
            write!(out, "{name}.{field} += ").unwrap();
            emit_expr(out, expr);
            out.push(';');
        }
    }
}

fn emit_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::Float(f) => write!(out, "{}", fmt_float(*f)).unwrap(),
        Expr::Ident(name) => out.push_str(name),
        Expr::Swizzle {name, field} => write!(out, "{name}.{field}").unwrap(),
        Expr::BinOp {op, left, right} => {
            out.push('(');
            emit_expr(out, left);
            write!(out, " {op} ").unwrap();
            emit_expr(out, right);
            out.push(')');
        }
        Expr::Call {name, args} => {
            out.push_str(name);
            out.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                emit_expr(out, arg);
            }
            out.push(')');
        }
    }
}

// Map a Gator type annotation to its plain GLSL equivalent
fn gator_to_glsl(ty: &str) -> String {
    match ty {
        "float" | "vec2" | "vec3" | "vec4" | "mat3" | "mat4" => ty.to_string(),
        _ => {
            let base = ty.split(['<', '.']).next().unwrap_or(ty);
            match base {
                "Cart3" | "Color3" => "vec3".to_string(),
                "Cart2" => "vec2".to_string(),
                "Color4" => "vec4".to_string(),
                "Hom4" => {
                    if ty.starts_with("Hom4.Matrix") || ty.starts_with("Hom4.InvTrMatrix") {
                        "mat4".to_string()
                    } else {
                        "vec4".to_string()
                    }
                }
                _ => ty.to_string()
            }
        }
    }
}

// Always emit a decimal point so GLSL treats the literal as float, not int
fn fmt_float(f: f32) -> String {
    if f.fract() == 0.0 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}
