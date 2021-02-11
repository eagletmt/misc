pub fn print_as_hcl2(miam: &crate::Miam) {
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

            print_policy_document(&format!("{}-{}", user.user_name, policy.name), &policy);
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

            print_policy_document(&format!("{}-{}", group.name, policy.name), &policy);
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

    for role in &miam.roles {
        println!(r#"resource "aws_iam_role" "{}" {{"#, role.name);
        println!(r#"  name = "{}""#, role.name);
        if let Some(ref path) = role.path {
            println!(r#"  path = "{}""#, path);
        }
        if role.assume_role_policy_document.is_some() {
            println!(
                "  assume_role_policy = data.aws_iam_policy_document.assume-role-{}.json",
                role.name
            );
        }
        if let Some(duration) = role.max_session_duration {
            println!("  max_session_duration = {}", duration);
        }
        println!("}}");

        if let Some(ref policy) = role.assume_role_policy_document {
            print_policy_document(&role.name, &policy);
        }

        for policy in &role.policies {
            println!(
                r#"resource "aws_iam_role_policy" "{}-{}" {{"#,
                role.name, policy.name
            );
            println!(r#"  name = "{}""#, policy.name);
            println!("  user = aws_iam_role.{}.name", role.name);
            println!(
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                role.name, policy.name
            );
            println!("}}");

            print_policy_document(&format!("{}-{}", role.name, policy.name), &policy);
        }
        for policy in &role.attached_managed_policies {
            let short_policy_name = policy.rsplitn(2, '/').next().unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} role",
                    policy, role.name
                )
            });
            println!(
                r#"resource "aws_iam_role_policy_attachment" "{}-{}" {{"#,
                role.name, short_policy_name
            );
            println!("  user = aws_iam_role.{}.name", role.name);
            println!("  policy_arn = aws_iam_policy.{}.arn", short_policy_name);
            println!("}}");
        }
    }

    for policy in &miam.managed_policies {
        println!(r#"resource "aws_iam_policy" "{}" {{"#, policy.name);
        println!(r#"  name = "{}""#, policy.name);
        if let Some(ref path) = policy.path {
            println!(r#"  path = "{}""#, path);
        }
        println!(
            "  policy = data.aws_iam_policy_document.{}.json",
            policy.name
        );
        println!("}}");

        print_policy_document(&policy.name, &policy.policy_document);
    }
}

fn print_policy_document(name: &str, policy_document: &crate::PolicyDocument) {
    println!(r#"data "aws_iam_policy_document" "{}" {{"#, name);
    if let Some(ref version) = policy_document.version {
        println!(r#"  version = "{}""#, version);
    }
    for statement in &policy_document.statements {
        println!(r#"  statement {{"#);
        println!(r#"    effect = "{}""#, statement.effect);
        println!("    actions = {:?}", statement.actions);
        println!(
            "    resources = {:?}",
            statement
                .resources
                .iter()
                .map(|s| replace_iam_interpolation(s))
                .collect::<Vec<_>>(),
        );
        for condition in &statement.conditions {
            println!("      condition {{");
            println!(r#"      test = "{}""#, condition.test);
            println!(r#"      variable = "{}""#, condition.variable);
            println!(
                "      values = {:?}",
                condition
                    .values
                    .iter()
                    .map(|s| replace_iam_interpolation(s))
                    .collect::<Vec<_>>()
            );
            println!("      }}");
        }
        println!("  }}");
    }
    println!("}}");
}

fn replace_iam_interpolation(s: &str) -> String {
    // https://registry.terraform.io/providers/hashicorp/aws/latest/docs/data-sources/iam_policy_document#context-variable-interpolation
    s.replace("${", "&{")
}
