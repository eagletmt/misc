use std::io::Read as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open("IAMfile")?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let mrb = unsafe { miam2tf::mruby::mrb_open() };
    let miam = unsafe {
        let dir_class = miam2tf::mruby::mrb_define_class(
            mrb,
            "Dir\0".as_ptr() as *const i8,
            (*mrb).object_class,
        );
        miam2tf::mruby::mrb_define_class_method(
            mrb,
            dir_class,
            "glob\0".as_ptr() as *const i8,
            Some(mrb_dir_glob),
            mrb_args_req(1),
        );
        miam2tf::mruby::mrb_define_singleton_method(
            mrb,
            (*mrb).top_self,
            "require\0".as_ptr() as *const i8,
            Some(mrb_require),
            mrb_args_req(1),
        );

        let ctx = miam2tf::mruby::mrbc_context_new(mrb);
        miam2tf::mruby::mrbc_filename(mrb, ctx, "IAMfile\0".as_ptr() as *const i8);
        miam2tf::mruby::mrb_load_nstring_cxt(
            mrb,
            code.as_ptr() as *const i8,
            code.len() as u64,
            ctx,
        );
        miam2tf::mruby::mrbc_context_free(mrb, ctx);
        if !(*mrb).exc.is_null() {
            miam2tf::mruby::mrb_print_error(mrb);
            std::process::exit(1);
        }

        let root = miam2tf::mruby::mrb_obj_iv_get(
            mrb,
            (*mrb).top_self,
            miam2tf::mruby::mrb_intern(mrb, "@root".as_ptr() as *const i8, "@root".len() as u64),
        );
        let miam = to_miam(mrb, root)?;
        miam2tf::mruby::mrb_close(mrb);
        miam
    };

    print_as_hcl2(&miam);
    Ok(())
}

fn attr_get(
    mrb: *mut miam2tf::mruby::mrb_state,
    obj: miam2tf::mruby::mrb_value,
    cstr: &'static str,
) -> miam2tf::mruby::mrb_value {
    unsafe { miam2tf::mruby::mrb_funcall(mrb, obj, cstr.as_ptr() as *const i8, 0) }
}

fn to_rust_string(mrb: *mut miam2tf::mruby::mrb_state, s: miam2tf::mruby::mrb_value) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(miam2tf::mruby::mrb_str_to_cstr(mrb, s))
            .to_string_lossy()
            .into_owned()
    }
}

struct ValueIter {
    idx: i64,
    len: i64,
    ary: miam2tf::mruby::mrb_value,
}
fn to_rust_iter(ary: miam2tf::mruby::mrb_value) -> ValueIter {
    ValueIter {
        idx: 0,
        len: rarray_len(ary),
        ary,
    }
}
impl std::iter::Iterator for ValueIter {
    type Item = miam2tf::mruby::mrb_value;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.idx < self.len {
            let val = unsafe { miam2tf::mruby::mrb_ary_entry(self.ary, self.idx) };
            self.idx += 1;
            Some(val)
        } else {
            None
        }
    }
}

fn mrb_args_req(n: i64) -> miam2tf::mruby::mrb_aspec {
    unsafe { miam2tf::mruby::wrapper_mrb_args_req(n) }
}

fn rarray_len(ary: miam2tf::mruby::mrb_value) -> i64 {
    unsafe { miam2tf::mruby::wrapper_rarray_len(ary) }
}

fn mrb_nil_p(o: miam2tf::mruby::mrb_value) -> bool {
    unsafe { miam2tf::mruby::wrapper_mrb_nil_p(o) != 0 }
}

fn e_runtime_error(mrb: *mut miam2tf::mruby::mrb_state) -> *mut miam2tf::mruby::RClass {
    unsafe { miam2tf::mruby::wrapper_e_runtime_error(mrb) }
}

fn mrb_nil_value() -> miam2tf::mruby::mrb_value {
    unsafe { miam2tf::mruby::wrapper_mrb_nil_value() }
}

fn unwrap_or_raise<T, E>(mrb: *mut miam2tf::mruby::mrb_state, r: Result<T, E>) -> T
where
    E: std::error::Error,
{
    match r {
        Ok(v) => v,
        Err(e) => {
            let msg = std::ffi::CString::new(format!("{:?}", e)).unwrap();
            unsafe { miam2tf::mruby::mrb_raise(mrb, e_runtime_error(mrb), msg.as_ptr()) };
            unreachable!();
        }
    }
}

extern "C" fn mrb_dir_glob(
    mrb: *mut miam2tf::mruby::mrb_state,
    _self: miam2tf::mruby::mrb_value,
) -> miam2tf::mruby::mrb_value {
    let mut block = mrb_nil_value();
    let pat = unsafe {
        let mut val = mrb_nil_value();
        miam2tf::mruby::mrb_get_args(mrb, "S&\0".as_ptr() as *const i8, &mut val, &mut block);
        to_rust_string(mrb, val)
    };

    if mrb_nil_p(block) {
        let entries = unsafe { miam2tf::mruby::mrb_ary_new(mrb) };
        for entry in unwrap_or_raise(mrb, glob::glob(&pat)) {
            let path = unwrap_or_raise(mrb, entry);
            let path_str = format!("{}", path.display());
            unsafe {
                let path_value = miam2tf::mruby::mrb_str_new(
                    mrb,
                    path_str.as_ptr() as *const i8,
                    path_str.len() as u64,
                );
                miam2tf::mruby::mrb_ary_push(mrb, entries, path_value);
            };
        }
        entries
    } else {
        for entry in unwrap_or_raise(mrb, glob::glob(&pat)) {
            let path = unwrap_or_raise(mrb, entry);
            let path_str = format!("{}", path.display());
            unsafe {
                let path_value = miam2tf::mruby::mrb_str_new(
                    mrb,
                    path_str.as_ptr() as *const i8,
                    path_str.len() as u64,
                );
                miam2tf::mruby::mrb_funcall(
                    mrb,
                    block,
                    "call\0".as_ptr() as *const i8,
                    1,
                    path_value,
                );
            };
        }
        mrb_nil_value()
    }
}

extern "C" fn mrb_require(
    mrb: *mut miam2tf::mruby::mrb_state,
    _self: miam2tf::mruby::mrb_value,
) -> miam2tf::mruby::mrb_value {
    let path = unsafe {
        let mut val = mrb_nil_value();
        miam2tf::mruby::mrb_get_args(mrb, "S\0".as_ptr() as *const i8, &mut val);
        to_rust_string(mrb, val)
    };
    let mut file = unwrap_or_raise(mrb, std::fs::File::open(&path));
    let mut code = Vec::new();
    unwrap_or_raise(mrb, file.read_to_end(&mut code));
    unsafe {
        let path_cstr = unwrap_or_raise(mrb, std::ffi::CString::new(path.as_bytes()));
        let ctx = miam2tf::mruby::mrbc_context_new(mrb);
        miam2tf::mruby::mrbc_filename(mrb, ctx, path_cstr.as_ptr());
        miam2tf::mruby::mrb_load_nstring_cxt(
            mrb,
            code.as_ptr() as *const i8,
            code.len() as u64,
            ctx,
        );
        miam2tf::mruby::mrbc_context_free(mrb, ctx);
    }
    mrb_nil_value()
}

#[derive(Debug, serde::Serialize)]
struct Miam {
    users: Vec<User>,
    groups: Vec<Group>,
}
#[derive(Debug, serde::Serialize)]
struct User {
    user_name: String,
    path: Option<String>,
    policies: Vec<PolicyDocument>,
    groups: Vec<String>,
    attached_managed_policies: Vec<String>,
}
#[derive(Debug, serde::Serialize)]
struct PolicyDocument {
    name: String,
    version: Option<String>,
    statements: Vec<PolicyStatement>,
}
#[derive(Debug, serde::Serialize)]
struct PolicyStatement {
    effect: String,
    actions: Vec<String>,
    resources: Vec<String>,
    conditions: Vec<PolicyCondition>,
}
#[derive(Debug, serde::Serialize)]
struct PolicyCondition {
    test: String,
    variable: String,
    values: Vec<String>,
}
#[derive(Debug, serde::Serialize)]
struct Group {
    name: String,
    path: Option<String>,
    policies: Vec<PolicyDocument>,
    attached_managed_policies: Vec<String>,
}

fn to_rust_policy_document(
    mrb: *mut miam2tf::mruby::mrb_state,
    policy: miam2tf::mruby::mrb_value,
) -> PolicyDocument {
    let name = to_rust_string(mrb, attr_get(mrb, policy, "name\0"));
    let version_value = attr_get(mrb, policy, "version\0");
    let version = if mrb_nil_p(version_value) {
        None
    } else {
        Some(to_rust_string(mrb, version_value))
    };
    let mut statements = Vec::new();
    for statement in to_rust_iter(attr_get(mrb, policy, "statements\0")) {
        let effect = to_rust_string(mrb, attr_get(mrb, statement, "effect\0"));
        let mut actions = Vec::new();
        for action in to_rust_iter(attr_get(mrb, statement, "actions\0")) {
            actions.push(to_rust_string(mrb, action));
        }
        let mut resources = Vec::new();
        for resource in to_rust_iter(attr_get(mrb, statement, "resources\0")) {
            resources.push(to_rust_string(mrb, resource));
        }
        let mut conditions = Vec::new();
        for condition in to_rust_iter(attr_get(mrb, statement, "conditions\0")) {
            let test = to_rust_string(mrb, attr_get(mrb, condition, "test\0"));
            let variable = to_rust_string(mrb, attr_get(mrb, condition, "variable\0"));
            let mut values = Vec::new();
            for value in to_rust_iter(attr_get(mrb, condition, "values\0")) {
                values.push(to_rust_string(mrb, value));
            }
            conditions.push(PolicyCondition {
                test,
                variable,
                values,
            });
        }
        statements.push(PolicyStatement {
            effect,
            actions,
            resources,
            conditions,
        });
    }
    PolicyDocument {
        name,
        version,
        statements,
    }
}

fn to_miam(
    mrb: *mut miam2tf::mruby::mrb_state,
    root: miam2tf::mruby::mrb_value,
) -> Result<Miam, Box<dyn std::error::Error>> {
    let mut users = Vec::new();
    for user in to_rust_iter(attr_get(mrb, root, "users\0")) {
        let user_name = to_rust_string(mrb, attr_get(mrb, user, "user_name\0"));
        let path = attr_get(mrb, user, "path\0");
        let path = if mrb_nil_p(path) {
            None
        } else {
            Some(to_rust_string(mrb, path))
        };
        let mut policies = Vec::new();
        for policy in to_rust_iter(attr_get(mrb, user, "policies\0")) {
            policies.push(to_rust_policy_document(mrb, policy));
        }
        let mut groups = Vec::new();
        for group in to_rust_iter(attr_get(mrb, user, "groups\0")) {
            groups.push(to_rust_string(mrb, group));
        }
        let mut attached_managed_policies = Vec::new();
        for policy in to_rust_iter(attr_get(mrb, user, "attached_managed_policies\0")) {
            attached_managed_policies.push(to_rust_string(mrb, policy));
        }
        users.push(User {
            user_name,
            path,
            policies,
            groups,
            attached_managed_policies,
        });
    }

    let mut groups = Vec::new();
    for group in to_rust_iter(attr_get(mrb, root, "groups\0")) {
        let name = to_rust_string(mrb, attr_get(mrb, group, "name\0"));
        eprintln!("processing {:?}", name);
        let path = attr_get(mrb, group, "path\0");
        let path = if mrb_nil_p(path) {
            None
        } else {
            Some(to_rust_string(mrb, path))
        };
        let mut policies = Vec::new();
        for policy in to_rust_iter(attr_get(mrb, group, "policies\0")) {
            eprintln!("to_rust_policy_document");
            policies.push(to_rust_policy_document(mrb, policy));
        }
        eprintln!("done policies");
        let mut attached_managed_policies = Vec::new();
        for policy in to_rust_iter(attr_get(mrb, group, "attached_managed_policies\0")) {
            attached_managed_policies.push(to_rust_string(mrb, policy));
        }
        groups.push(Group {
            name,
            path,
            policies,
            attached_managed_policies,
        });
    }
    Ok(Miam { users, groups })
}

fn print_as_hcl2(miam: &Miam) {
    for user in &miam.users {
        println!(r#"resource "aws_iam_user" "{}" {{"#, user.user_name);
        println!(r#"  name = "{}""#, user.user_name);
        if let Some(ref path) = user.path {
            println!(r#"  path = "{}""#, path);
        }
        println!("}}");

        for policy in &user.policies {
            println!(
                r#"resource "aws_iam_user_policy" "{}-{}" {{"#,
                user.user_name, policy.name
            );
            println!(r#"  name = "{}""#, policy.name);
            println!("  user = aws_iam_user.{}.name", user.user_name);
            println!(
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                user.user_name, policy.name
            );
            println!("}}");

            println!(
                r#"data "aws_iam_policy_document" "{}-{}" {{"#,
                user.user_name, policy.name
            );
            if let Some(ref version) = policy.version {
                println!(r#"  version = "{}""#, version);
            }
            for statement in &policy.statements {
                println!(r#"  statement {{"#);
                println!(r#"    effect = "{}""#, statement.effect);
                println!("    actions = {:?}", statement.actions);
                println!("    resources = {:?}", statement.resources);
                for condition in &statement.conditions {
                    println!("      condition {{");
                    println!(r#"      test = "{}""#, condition.test);
                    println!(r#"      variable = "{}""#, condition.variable);
                    println!("      values = {:?}", condition.values);
                    println!("      }}");
                }
                println!("  }}");
            }
            println!("}}");
        }
        if !user.groups.is_empty() {
            println!(
                r#"resource "aws_iam_user_group_membership" "{}" {{"#,
                user.user_name,
            );
            println!("  user = aws_iam_user.{}.name", user.user_name);
            println!("  groups = [");
            for group in &user.groups {
                println!("    aws_iam_group.{}.name,", group);
            }
            println!("  ]");
            println!("}}");
        }
        for policy in &user.attached_managed_policies {
            let short_policy_name = policy.rsplitn(2, '/').next().unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} user",
                    policy, user.user_name
                )
            });
            println!(
                r#"resource "aws_iam_user_policy_attachment" "{}-{}" {{"#,
                user.user_name, short_policy_name
            );
            println!("  user = aws_iam_user.{}.name", user.user_name);
            println!("  policy_arn = aws_iam_policy.{}.arn", short_policy_name);
            println!("}}");
        }
    }

    for group in &miam.groups {
        println!(r#"resource "aws_iam_group" "{}" {{"#, group.name);
        println!(r#"  name = "{}""#, group.name);
        if let Some(ref path) = group.path {
            println!(r#"  path = "{}""#, path);
        }
        println!("}}");

        for policy in &group.policies {
            println!(
                r#"resource "aws_iam_group_policy" "{}-{}" {{"#,
                group.name, policy.name
            );
            println!(r#"  name = "{}""#, policy.name);
            println!("  user = aws_iam_group.{}.name", group.name);
            println!(
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                group.name, policy.name
            );
            println!("}}");

            println!(
                r#"data "aws_iam_policy_document" "{}-{}" {{"#,
                group.name, policy.name
            );
            if let Some(ref version) = policy.version {
                println!(r#"  version = "{}""#, version);
            }
            for statement in &policy.statements {
                println!(r#"  statement {{"#);
                println!(r#"    effect = "{}""#, statement.effect);
                println!("    actions = {:?}", statement.actions);
                println!("    resources = {:?}", statement.resources);
                for condition in &statement.conditions {
                    println!("      condition {{");
                    println!(r#"      test = "{}""#, condition.test);
                    println!(r#"      variable = "{}""#, condition.variable);
                    println!("      values = {:?}", condition.values);
                    println!("      }}");
                }
                println!("  }}");
            }
            println!("}}");
        }
        for policy in &group.attached_managed_policies {
            let short_policy_name = policy.rsplitn(2, '/').next().unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} group",
                    policy, group.name
                )
            });
            println!(
                r#"resource "aws_iam_group_policy_attachment" "{}-{}" {{"#,
                group.name, short_policy_name
            );
            println!("  user = aws_iam_group.{}.name", group.name);
            println!("  policy_arn = aws_iam_policy.{}.arn", short_policy_name);
            println!("}}");
        }
    }
}
