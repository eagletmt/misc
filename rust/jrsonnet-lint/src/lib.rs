#[derive(Debug)]
pub struct Variable {
    name: String,
    path: std::path::PathBuf,
    begin_offset: usize,
    end_offset: usize,
}

pub fn find_unused_variables(expr: &jrsonnet_parser::LocExpr) -> Vec<Variable> {
    let mut env = std::collections::HashMap::new();
    env.insert("std".to_owned(), 0);
    let mut counter = env.len() as isize;
    let simplified = simplify_expr(expr, &env, &mut counter);
    let mut unused_variables = Vec::new();
    find_unused(
        &mut unused_variables,
        &simplified,
        &mut std::collections::HashSet::new(),
    );
    unused_variables
        .into_iter()
        .map(|(name, location)| Variable {
            name,
            path: location.0.to_path_buf(),
            begin_offset: location.1,
            end_offset: location.2,
        })
        .collect()
}

#[derive(Debug)]
enum Simplified {
    Bind {
        location: jrsonnet_parser::ExprLocation,
        name: String,
        index: isize,
        expr: Box<Simplified>,
        child: Box<Simplified>,
    },
    Expr {
        children: Vec<Simplified>,
    },
    Var {
        index: isize,
    },
    Lit,
}

fn simplify_expr(
    loc_expr: &jrsonnet_parser::LocExpr,
    env: &std::collections::HashMap<String, isize>,
    counter: &mut isize,
) -> Simplified {
    match loc_expr.0.as_ref() {
        jrsonnet_parser::Expr::Literal(_)
        | jrsonnet_parser::Expr::Str(_)
        | jrsonnet_parser::Expr::Num(_) => Simplified::Lit,
        jrsonnet_parser::Expr::Var(var_name) => {
            if let Some(index) = env.get(var_name as &str) {
                Simplified::Var { index: *index }
            } else {
                // unbound variable
                Simplified::Var { index: -1 }
            }
        }
        jrsonnet_parser::Expr::LocalExpr(bind_specs, child_expr) => {
            let mut next_env = env.clone();
            let binds = simplify_binds(bind_specs, &mut next_env, counter);
            let mut simplified = simplify_expr(child_expr, &next_env, counter);
            for (name, index, expr) in binds.into_iter().rev() {
                simplified = Simplified::Bind {
                    location: loc_expr.1.to_owned().unwrap(),
                    name,
                    index,
                    expr: Box::new(expr),
                    child: Box::new(simplified),
                };
            }
            simplified
        }
        jrsonnet_parser::Expr::Obj(obj_body) => simplify_obj_body(loc_expr, obj_body, env, counter),
        jrsonnet_parser::Expr::UnaryOp(_, e) => simplify_expr(e, env, counter),
        jrsonnet_parser::Expr::BinaryOp(lhs, _, rhs) => Simplified::Expr {
            children: vec![
                simplify_expr(lhs, env, counter),
                simplify_expr(rhs, env, counter),
            ],
        },
        jrsonnet_parser::Expr::Apply(func, jrsonnet_parser::ArgsDesc(args), _) => {
            let mut children = Vec::with_capacity(args.len() + 1);
            children.push(simplify_expr(func, env, counter));
            for jrsonnet_parser::Arg(_, arg) in args {
                children.push(simplify_expr(arg, env, counter));
            }
            Simplified::Expr { children }
        }
        jrsonnet_parser::Expr::Index(obj, idx) => Simplified::Expr {
            children: vec![
                simplify_expr(obj, env, counter),
                simplify_expr(idx, env, counter),
            ],
        },
        jrsonnet_parser::Expr::ArrComp(child_expr, comp_specs) => {
            let mut children = Vec::new();
            enum Spec {
                For {
                    name: String,
                    index: isize,
                    s: Simplified,
                },
                If {
                    s: Simplified,
                },
            }

            let mut next_env = env.clone();
            for comp_spec in comp_specs {
                match comp_spec {
                    jrsonnet_parser::CompSpec::IfSpec(jrsonnet_parser::IfSpecData(if_expr)) => {
                        children.push(Spec::If {
                            s: simplify_expr(if_expr, &next_env, counter),
                        });
                    }
                    jrsonnet_parser::CompSpec::ForSpec(jrsonnet_parser::ForSpecData(
                        var_name,
                        for_expr,
                    )) => {
                        let name = var_name.to_string();
                        let index = *counter;
                        *counter += 1;
                        next_env.insert(name.clone(), index);
                        let s = simplify_expr(for_expr, &next_env, counter);
                        children.push(Spec::For { name, index, s });
                    }
                }
            }
            let mut simplified = simplify_expr(child_expr, &next_env, counter);
            let mut v = Vec::new();
            for child in children.into_iter().rev() {
                match child {
                    Spec::For { name, index, s } => {
                        simplified = if v.is_empty() {
                            Simplified::Bind {
                                location: loc_expr.1.to_owned().unwrap(),
                                name,
                                index,
                                expr: Box::new(s),
                                child: Box::new(simplified),
                            }
                        } else {
                            v.push(simplified);
                            Simplified::Bind {
                                location: loc_expr.1.to_owned().unwrap(),
                                name,
                                index,
                                expr: Box::new(s),
                                child: Box::new(Simplified::Expr { children: v }),
                            }
                        };
                        v = Vec::new();
                    }
                    Spec::If { s } => {
                        v.push(s);
                    }
                }
            }
            assert!(v.is_empty());
            simplified
        }
        jrsonnet_parser::Expr::Arr(loc_exprs) => Simplified::Expr {
            children: loc_exprs
                .iter()
                .map(|e| simplify_expr(e, env, counter))
                .collect(),
        },
        jrsonnet_parser::Expr::ObjExtend(e, obj_body) => Simplified::Expr {
            children: vec![
                simplify_expr(e, env, counter),
                simplify_obj_body(loc_expr, obj_body, env, counter),
            ],
        },
        jrsonnet_parser::Expr::Parened(e) => simplify_expr(e, env, counter),
        jrsonnet_parser::Expr::Import(_) => Simplified::Lit,
        jrsonnet_parser::Expr::ImportStr(_) => Simplified::Lit,
        jrsonnet_parser::Expr::AssertExpr(
            jrsonnet_parser::AssertStmt(assert_expr, assert_message),
            child,
        ) => {
            if let Some(assert_message) = assert_message {
                Simplified::Expr {
                    children: vec![
                        simplify_expr(assert_expr, env, counter),
                        simplify_expr(assert_message, env, counter),
                        simplify_expr(child, env, counter),
                    ],
                }
            } else {
                Simplified::Expr {
                    children: vec![
                        simplify_expr(assert_expr, env, counter),
                        simplify_expr(child, env, counter),
                    ],
                }
            }
        }
        jrsonnet_parser::Expr::ErrorStmt(e) => simplify_expr(e, env, counter),
        jrsonnet_parser::Expr::Intrinsic(_) => {
            // Should not happen in parsing normal Jsonnet
            Simplified::Lit
        }
        jrsonnet_parser::Expr::IfElse {
            cond: jrsonnet_parser::IfSpecData(cond),
            cond_then,
            cond_else,
        } => {
            if let Some(cond_else) = cond_else {
                Simplified::Expr {
                    children: vec![
                        simplify_expr(cond, env, counter),
                        simplify_expr(cond_then, env, counter),
                        simplify_expr(cond_else, env, counter),
                    ],
                }
            } else {
                Simplified::Expr {
                    children: vec![
                        simplify_expr(cond, env, counter),
                        simplify_expr(cond_then, env, counter),
                    ],
                }
            }
        }
        jrsonnet_parser::Expr::Slice(e, slice_desc) => {
            let mut children = vec![simplify_expr(e, env, counter)];
            if let Some(start) = &slice_desc.start {
                children.push(simplify_expr(start, env, counter));
            }
            if let Some(end) = &slice_desc.end {
                children.push(simplify_expr(end, env, counter));
            }
            if let Some(step) = &slice_desc.step {
                children.push(simplify_expr(step, env, counter));
            }
            Simplified::Expr { children }
        }
        jrsonnet_parser::Expr::Function(params, body) => {
            simplify_func(loc_expr, params, body, env, counter)
        }
    }
}

fn simplify_obj_body(
    loc_expr: &jrsonnet_parser::LocExpr,
    obj_body: &jrsonnet_parser::ObjBody,
    env: &std::collections::HashMap<String, isize>,
    counter: &mut isize,
) -> Simplified {
    match obj_body {
        jrsonnet_parser::ObjBody::MemberList(members) => {
            let mut bind_specs = Vec::new();
            for member in members {
                if let jrsonnet_parser::Member::BindStmt(bind_spec) = member {
                    bind_specs.push(bind_spec.to_owned());
                }
            }
            let mut next_env = env.clone();
            let binds = simplify_binds(&bind_specs, &mut next_env, counter);

            let mut children = Vec::with_capacity(members.len() - binds.len());
            for member in members {
                match member {
                    jrsonnet_parser::Member::Field(field_member) => {
                        if let jrsonnet_parser::FieldName::Dyn(field_name_expr) = &field_member.name
                        {
                            children.push(simplify_expr(field_name_expr, &next_env, counter));
                        }
                        if let Some(params) = &field_member.params {
                            children.push(simplify_func(
                                loc_expr,
                                params,
                                &field_member.value,
                                &next_env,
                                counter,
                            ));
                        } else {
                            children.push(simplify_expr(&field_member.value, &next_env, counter));
                        }
                    }
                    jrsonnet_parser::Member::AssertStmt(jrsonnet_parser::AssertStmt(
                        assert_expr,
                        assert_message,
                    )) => {
                        if let Some(assert_message) = assert_message {
                            children.push(Simplified::Expr {
                                children: vec![
                                    simplify_expr(assert_expr, &next_env, counter),
                                    simplify_expr(assert_message, &next_env, counter),
                                ],
                            });
                        } else {
                            children.push(Simplified::Expr {
                                children: vec![simplify_expr(assert_expr, &next_env, counter)],
                            });
                        }
                    }
                    jrsonnet_parser::Member::BindStmt(_) => {
                        // already handled
                    }
                }
            }

            let mut simplified = Simplified::Expr { children };
            for (name, index, s) in binds.into_iter().rev() {
                simplified = Simplified::Bind {
                    location: loc_expr.1.to_owned().unwrap(),
                    name,
                    index,
                    expr: Box::new(s),
                    child: Box::new(simplified),
                };
            }
            simplified
        }
        jrsonnet_parser::ObjBody::ObjComp(obj_comp) => {
            let mut children = Vec::new();
            enum Spec {
                For {
                    name: String,
                    index: isize,
                    s: Simplified,
                },
                If {
                    s: Simplified,
                },
            }

            let mut next_env = env.clone();
            for comp_spec in &obj_comp.compspecs {
                match comp_spec {
                    jrsonnet_parser::CompSpec::IfSpec(jrsonnet_parser::IfSpecData(if_expr)) => {
                        children.push(Spec::If {
                            s: simplify_expr(if_expr, &next_env, counter),
                        });
                    }
                    jrsonnet_parser::CompSpec::ForSpec(jrsonnet_parser::ForSpecData(
                        var_name,
                        for_expr,
                    )) => {
                        let name = var_name.to_string();
                        let index = *counter;
                        *counter += 1;
                        next_env.insert(name.clone(), index);
                        let s = simplify_expr(for_expr, &next_env, counter);
                        children.push(Spec::For { name, index, s });
                    }
                }
            }
            let pre_binds = simplify_binds(&obj_comp.pre_locals, &mut next_env, counter);
            let post_binds = simplify_binds(&obj_comp.post_locals, &mut next_env, counter);
            let mut binds = pre_binds;
            binds.extend(post_binds);

            let mut simplified = Simplified::Expr {
                children: vec![
                    simplify_expr(&obj_comp.key, &next_env, counter),
                    simplify_expr(&obj_comp.value, &next_env, counter),
                ],
            };
            for (name, index, s) in binds.into_iter().rev() {
                simplified = Simplified::Bind {
                    location: loc_expr.1.to_owned().unwrap(),
                    name,
                    index,
                    expr: Box::new(s),
                    child: Box::new(simplified),
                };
            }

            let mut v = Vec::new();
            for child in children.into_iter().rev() {
                match child {
                    Spec::For { name, index, s } => {
                        simplified = if v.is_empty() {
                            Simplified::Bind {
                                location: loc_expr.1.to_owned().unwrap(),
                                name,
                                index,
                                expr: Box::new(s),
                                child: Box::new(simplified),
                            }
                        } else {
                            v.push(simplified);
                            Simplified::Bind {
                                location: loc_expr.1.to_owned().unwrap(),
                                name,
                                index,
                                expr: Box::new(s),
                                child: Box::new(Simplified::Expr { children: v }),
                            }
                        };
                        v = Vec::new();
                    }
                    Spec::If { s } => {
                        v.push(s);
                    }
                }
            }
            assert!(v.is_empty());
            simplified
        }
    }
}

fn simplify_func(
    loc_expr: &jrsonnet_parser::LocExpr,
    params: &jrsonnet_parser::ParamsDesc,
    body: &jrsonnet_parser::LocExpr,
    env: &std::collections::HashMap<String, isize>,
    counter: &mut isize,
) -> Simplified {
    let mut next_env = env.clone();
    let mut binds = Vec::with_capacity(params.len());
    for param in params.iter() {
        let name = param.0.to_string();
        let index = *counter;
        *counter += 1;
        let child = if let Some(default_expr) = &param.1 {
            simplify_expr(default_expr, &next_env, counter)
        } else {
            Simplified::Lit
        };
        next_env.insert(name.clone(), index);
        binds.push((name, index, child));
    }
    let mut simplified = simplify_expr(body, &next_env, counter);
    for (name, index, expr) in binds.into_iter().rev() {
        simplified = Simplified::Bind {
            location: loc_expr.1.to_owned().unwrap(),
            name,
            index,
            expr: Box::new(expr),
            child: Box::new(simplified),
        }
    }
    simplified
}

fn simplify_binds(
    bind_specs: &[jrsonnet_parser::BindSpec],
    next_env: &mut std::collections::HashMap<String, isize>,
    counter: &mut isize,
) -> Vec<(String, isize, Simplified)> {
    let mut binds = Vec::with_capacity(bind_specs.len());
    for bind_spec in bind_specs {
        if let Some(params) = &bind_spec.params {
            for param in params.iter() {
                let name = param.0.to_string();
                let index = *counter;
                *counter += 1;
                let child = if let Some(default_expr) = &param.1 {
                    simplify_expr(default_expr, next_env, counter)
                } else {
                    Simplified::Lit
                };
                next_env.insert(name.clone(), index);
                binds.push((name, index, child));
            }
        }
        let name = bind_spec.name.to_string();
        let index = *counter;
        *counter += 1;
        let child = simplify_expr(&bind_spec.value, next_env, counter);
        next_env.insert(name.clone(), index);
        binds.push((name, index, child));
    }
    binds
}

fn find_unused(
    unused_variables: &mut Vec<(String, jrsonnet_parser::ExprLocation)>,
    expr: &Simplified,
    bound_indices: &mut std::collections::HashSet<isize>,
) {
    match expr {
        Simplified::Expr { children } => {
            for child in children {
                find_unused(unused_variables, child, bound_indices);
            }
        }
        Simplified::Bind {
            location,
            name,
            index,
            expr,
            child,
        } => {
            find_unused(unused_variables, expr, bound_indices);
            bound_indices.insert(*index);
            find_unused(unused_variables, child, bound_indices);
            if bound_indices.contains(index) {
                unused_variables.push((name.to_owned(), location.to_owned()));
            }
        }
        Simplified::Var { index } => {
            bound_indices.remove(index);
        }
        Simplified::Lit => {}
    }
}

#[cfg(test)]
mod tests {
    fn doit(code: &str) -> Vec<super::Variable> {
        let expr = jrsonnet_parser::parse(
            &code,
            &jrsonnet_parser::ParserSettings {
                loc_data: true,
                file_name: std::path::PathBuf::from("test.jsonnet").into(),
            },
        )
        .expect("failed to parse Jsonnet");
        super::find_unused_variables(&expr)
    }

    #[test]
    fn no_variables() {
        let vs = doit("{ x: 1 }");
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn use_variable() {
        let vs = doit(indoc::indoc! {"
            local x = 1;
            { x: x }
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn unused_variable() {
        let vs = doit(indoc::indoc! {"
            local x = 1;
            local y = 2;
            { x: x }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "y");
        assert_eq!(vs[0].begin_offset, 13);
    }

    #[test]
    fn unbound_variable() {
        let vs = doit(indoc::indoc! {"
            { x: x }
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn multiple_local_binds() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = std.toString(x);
            { x: 0 }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "y");
        assert_eq!(vs[0].begin_offset, 0);
    }

    #[test]
    fn used_in_dynamic_field_name() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = std.toString(x);
            { [y]: 0 }
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn used_in_field_function_default_parameter() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2;
            {
                f(n=x):: n,
                y: self.f(1),
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "y");
    }

    #[test]
    fn used_in_field_local_function_default_parameter() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2, f(m, n=y) = n + x;
            {
                z: f(1, 2),
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "m");
    }

    #[test]
    fn array_comprehension() {
        let vs = doit(indoc::indoc! {"
            [
                [j]
                for i in std.range(1, 10)
                if i % 2 == 0
                for j in std.range(1, i)
            ]
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn unused_array_comprehension() {
        let vs = doit(indoc::indoc! {"
            [
                [i]
                for i in std.range(1, 10)
                if i % 2 == 0
                if i % 3 == 0
                for j in std.range(1, i)
            ]
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "j");
    }

    #[test]
    fn obj_extend() {
        let vs = doit(indoc::indoc! {"
            local o = { x: 1 }, x = -1, y = 2;
            o {
                y: y,
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "x");
    }

    #[test]
    fn parened() {
        let vs = doit(indoc::indoc! {"
            local o = { x: 1 }, x = 1, y = 2;
            o {
                y: (y),
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "x");
    }

    #[test]
    fn assert_without_message() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2, z = 3;
            assert x == 1;
            y
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "z");
    }

    #[test]
    fn assert_with_message() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2, z = 3;
            assert x == 1 : std.format('assertion failed %d', [z]);
            y
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn error_stmt() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2;
            error std.format('broken %d', [x])
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "y");
    }

    #[test]
    fn if_else() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2, z = 3, w = 4;
            if x == 1 then error std.format('error %d', [y]) else z
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "w");
    }

    #[test]
    fn if_then() {
        let vs = doit(indoc::indoc! {"
            local x = 1, y = 2, z = 3, w = 4;
            if x == 1 then error std.format('error %d', [y])
        "});
        assert_eq!(vs.len(), 2, "{:?}", vs);
        let mut unused_names: Vec<_> = vs.into_iter().map(|v| v.name).collect();
        unused_names.sort();
        assert_eq!(unused_names, ["w", "z"]);
    }

    #[test]
    fn slice() {
        let vs = doit(indoc::indoc! {"
            local x = 3, y = 6, z = 2, w = 0, u = 1;
            {
              x: std.range(w, 10)[1:x],
              y: std.range(w, 10)[2:y:2],
              z: std.range(w, 10)[2::z],
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "u");
    }

    #[test]
    fn func() {
        let vs = doit(indoc::indoc! {"
            local n = 1, m = 2, i = 3, f = function(x) x + n;

            f(i)
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "m");
    }

    #[test]
    fn object_comprehension() {
        let vs = doit(indoc::indoc! {"
            local x = 'x';
            local y = 2;
            local z = 10;
            local w = 'w';

            {
              local t = i,
              [x + i + j]: u + y,
              local u = t
              for i in std.range(1, z)
              if i % 2 == 0
              for j in std.range(1, 10)
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "w");
    }

    #[test]
    fn obj_member_bind() {
        let vs = doit(indoc::indoc! {"
            local x = error 'unreachable';

            {
              x: x,
              local x = 1,
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "x");
        assert_eq!(vs[0].begin_offset, 0);
    }

    #[test]
    fn obj_member_assert() {
        let vs = doit(indoc::indoc! {"
            local x = error 'unreachable';
            local y = 'message';

            {
              x: x,
              assert x == 1,
              assert x == 1 : y,
            }
        "});
        assert!(vs.is_empty(), "{:?}", vs);
    }

    #[test]
    fn ignore_imports() {
        let vs = doit(indoc::indoc! {"
            local x = import 'foo.libsonnet';
            local y = importstr 'bar.txt';

            {
              x: x,
            }
        "});
        assert_eq!(vs.len(), 1, "{:?}", vs);
        assert_eq!(vs[0].name, "y");
    }
}
