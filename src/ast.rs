use pest::iterators::Pairs;
use crate::Rule;

// Every kind of value-producing thing in the shader
#[derive(Debug)]
pub enum Expr {
    Float(f32),
    Ident(String),
    Swizzle {name: String, field: String},
    BinOp {op: char, left: Box<Expr>, right: Box<Expr>},
    Call {name: String, args: Vec<Expr>}
}

// A single line inside main()
#[derive(Debug)]
pub enum Stmt {
    Assign {name: String, expr: Expr},
    Decl {name: String, expr: Expr},
    SwizzleAssign {name: String, field: String, expr: Expr} // e.g. color.g += expr
}

// The whole program — the statements inside main(), everything else is discarded
#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>
}

// Walk Pest's raw output and pull out the statements from main()
pub fn parse_program(mut pairs: Pairs<Rule>) -> Program {
    let mut stmts = Vec::new();
    // Pairs contains one top-level 'program' node, so go into its children to find main_fn
    let program = pairs.next().unwrap();
    for pair in program.into_inner() {
        if pair.as_rule() == Rule::main_fn {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::stmt {
                    stmts.push(parse_stmt(inner));
                }
            }
        }
    }
    Program {stmts}
}

// A stmt node always contains either an assign_stmt, decl_stmt, or compound_assign_stmt
fn parse_stmt(pair: pest::iterators::Pair<Rule>) -> Stmt {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::assign_stmt => {
            let mut parts = inner.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let expr = parse_expr(parts.next().unwrap());
            Stmt::Assign {name, expr}
        }
        Rule::decl_stmt => {
            let mut parts = inner.into_inner();
            // Skip type_name
            parts.next();
            let name = parts.next().unwrap().as_str().to_string();
            let expr = parse_expr(parts.next().unwrap());
            Stmt::Decl {name, expr}
        }
        Rule::compound_assign_stmt => {
            let mut parts = inner.into_inner();
            // Ident
            let name = parts.next().unwrap().as_str().to_string();
            // Swizzle
            let field = parts.next().unwrap().as_str().to_string();
            let expr = parse_expr(parts.next().unwrap());
            Stmt::SwizzleAssign {name, field, expr}
        }
        _ => unreachable!()
    }
}

// Recursively build an Expr from Pest nodes
// The grammar layers (expr -> add_expr -> mul_expr -> unary -> primary) enforce
// precedence, so just collapse each layer
fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => parse_expr(pair.into_inner().next().unwrap()),
        Rule::add_expr => {
            // Children alternate: [mul_expr, add_op, mul_expr, add_op, ...]
            let mut children: Vec<_> = pair.into_inner().collect();
            let mut left = parse_expr(children.remove(0));
            while children.len() >= 2 {
                let op = children.remove(0).as_str().chars().next().unwrap();
                let right = parse_expr(children.remove(0));
                left = Expr::BinOp {op, left: Box::new(left), right: Box::new(right)};
            }
            left
        }
        Rule::mul_expr => {
            let mut children: Vec<_> = pair.into_inner().collect();
            let mut left = parse_expr(children.remove(0));
            while children.len() >= 2 {
                let op = children.remove(0).as_str().chars().next().unwrap();
                let right = parse_expr(children.remove(0));
                left = Expr::BinOp {op, left: Box::new(left), right: Box::new(right)};
            }
            left
        }
        Rule::unary => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            // If first token is "-", negate by multiplying by -1
            if first.as_str() == "-" {
                let operand = parse_expr(inner.next().unwrap());
                Expr::BinOp {
                    op: '*',
                    left: Box::new(Expr::Float(-1.0)),
                    right: Box::new(operand)
                }
            } else {
                parse_expr(first)
            }
        }
        Rule::primary => parse_expr(pair.into_inner().next().unwrap()),
        Rule::call_expr => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let args = inner.map(parse_expr).collect();
            Expr::Call {name, args}
        }
        Rule::swizzle_expr => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let field = inner.next().unwrap().as_str().to_string();
            Expr::Swizzle {name, field}
        }
        Rule::float_lit => Expr::Float(pair.as_str().parse().unwrap()),
        Rule::ident => Expr::Ident(pair.as_str().to_string()),
        Rule::add_op | Rule::mul_op => unreachable!("ops are consumed by add_expr/mul_expr"),
        _ => unreachable!("{:?}", pair.as_rule())
    }
}
