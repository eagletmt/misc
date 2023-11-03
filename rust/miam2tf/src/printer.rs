pub fn print_as_hcl2<W>(writer: &mut W, miam: &crate::Miam) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    for user in &miam.users {
        writeln!(writer, r#"resource "aws_iam_user" "{}" {{"#, user.user_name)?;
        writeln!(writer, r#"  name = "{}""#, user.user_name)?;
        if let Some(ref path) = user.path {
            writeln!(writer, r#"  path = "{}""#, path)?;
        }
        writeln!(writer, "}}")?;

        for policy in &user.policies {
            writeln!(
                writer,
                r#"resource "aws_iam_user_policy" "{}-{}" {{"#,
                user.user_name, policy.name
            )?;
            writeln!(writer, r#"  name = "{}""#, policy.name)?;
            writeln!(writer, "  user = aws_iam_user.{}.name", user.user_name)?;
            writeln!(
                writer,
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                user.user_name, policy.name
            )?;
            writeln!(writer, "}}")?;

            print_policy_document(
                writer,
                &format!("{}-{}", user.user_name, policy.name),
                policy,
            )?;
        }
        if !user.groups.is_empty() {
            writeln!(
                writer,
                r#"resource "aws_iam_user_group_membership" "{}" {{"#,
                user.user_name,
            )?;
            writeln!(writer, "  user = aws_iam_user.{}.name", user.user_name)?;
            writeln!(writer, "  groups = [")?;
            for group in &user.groups {
                writeln!(writer, "    aws_iam_group.{}.name,", group)?;
            }
            writeln!(writer, "  ]")?;
            writeln!(writer, "}}")?;
        }
        for policy in &user.attached_managed_policies {
            let short_policy_name = policy.rsplit_once('/').map(|(_, x)| x).unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} user",
                    policy, user.user_name
                )
            });
            writeln!(
                writer,
                r#"resource "aws_iam_user_policy_attachment" "{}-{}" {{"#,
                user.user_name, short_policy_name
            )?;
            writeln!(writer, "  user = aws_iam_user.{}.name", user.user_name)?;
            writeln!(
                writer,
                "  policy_arn = aws_iam_policy.{}.arn",
                short_policy_name
            )?;
            writeln!(writer, "}}")?;
        }
    }

    for group in &miam.groups {
        writeln!(writer, r#"resource "aws_iam_group" "{}" {{"#, group.name)?;
        writeln!(writer, r#"  name = "{}""#, group.name)?;
        if let Some(ref path) = group.path {
            writeln!(writer, r#"  path = "{}""#, path)?;
        }
        writeln!(writer, "}}")?;

        for policy in &group.policies {
            writeln!(
                writer,
                r#"resource "aws_iam_group_policy" "{}-{}" {{"#,
                group.name, policy.name
            )?;
            writeln!(writer, r#"  name = "{}""#, policy.name)?;
            writeln!(writer, "  user = aws_iam_group.{}.name", group.name)?;
            writeln!(
                writer,
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                group.name, policy.name
            )?;
            writeln!(writer, "}}")?;

            print_policy_document(writer, &format!("{}-{}", group.name, policy.name), policy)?;
        }
        for policy in &group.attached_managed_policies {
            let short_policy_name = policy.rsplit_once('/').map(|(_, x)| x).unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} group",
                    policy, group.name
                )
            });
            writeln!(
                writer,
                r#"resource "aws_iam_group_policy_attachment" "{}-{}" {{"#,
                group.name, short_policy_name
            )?;
            writeln!(writer, "  user = aws_iam_group.{}.name", group.name)?;
            writeln!(
                writer,
                "  policy_arn = aws_iam_policy.{}.arn",
                short_policy_name
            )?;
            writeln!(writer, "}}")?;
        }
    }

    for role in &miam.roles {
        writeln!(writer, r#"resource "aws_iam_role" "{}" {{"#, role.name)?;
        writeln!(writer, r#"  name = "{}""#, role.name)?;
        if let Some(ref path) = role.path {
            writeln!(writer, r#"  path = "{}""#, path)?;
        }
        if role.assume_role_policy_document.is_some() {
            writeln!(
                writer,
                "  assume_role_policy = data.aws_iam_policy_document.assume-role-{}.json",
                role.name
            )?;
        }
        if let Some(duration) = role.max_session_duration {
            writeln!(writer, "  max_session_duration = {}", duration)?;
        }
        writeln!(writer, "}}")?;

        if let Some(ref policy) = role.assume_role_policy_document {
            print_policy_document(writer, &role.name, policy)?;
        }

        for policy in &role.policies {
            writeln!(
                writer,
                r#"resource "aws_iam_role_policy" "{}-{}" {{"#,
                role.name, policy.name
            )?;
            writeln!(writer, r#"  name = "{}""#, policy.name)?;
            writeln!(writer, "  user = aws_iam_role.{}.name", role.name)?;
            writeln!(
                writer,
                "  policy = data.aws_iam_policy_document.{}-{}.json",
                role.name, policy.name
            )?;
            writeln!(writer, "}}")?;

            print_policy_document(writer, &format!("{}-{}", role.name, policy.name), policy)?;
        }
        for policy in &role.attached_managed_policies {
            let short_policy_name = policy.rsplit_once('/').map(|(_, x)| x).unwrap_or_else(|| {
                panic!(
                    "Invalid attached_managed_policies {} found in {} role",
                    policy, role.name
                )
            });
            writeln!(
                writer,
                r#"resource "aws_iam_role_policy_attachment" "{}-{}" {{"#,
                role.name, short_policy_name
            )?;
            writeln!(writer, "  user = aws_iam_role.{}.name", role.name)?;
            writeln!(
                writer,
                "  policy_arn = aws_iam_policy.{}.arn",
                short_policy_name
            )?;
            writeln!(writer, "}}")?;
        }
    }

    for policy in &miam.managed_policies {
        writeln!(writer, r#"resource "aws_iam_policy" "{}" {{"#, policy.name)?;
        writeln!(writer, r#"  name = "{}""#, policy.name)?;
        if let Some(ref path) = policy.path {
            writeln!(writer, r#"  path = "{}""#, path)?;
        }
        writeln!(
            writer,
            "  policy = data.aws_iam_policy_document.{}.json",
            policy.name
        )?;
        writeln!(writer, "}}")?;

        print_policy_document(writer, &policy.name, &policy.policy_document)?;
    }
    Ok(())
}

fn print_policy_document<W>(
    writer: &mut W,
    name: &str,
    policy_document: &crate::PolicyDocument,
) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    writeln!(writer, r#"data "aws_iam_policy_document" "{}" {{"#, name)?;
    if let Some(ref version) = policy_document.version {
        writeln!(writer, r#"  version = "{}""#, version)?;
    }
    for statement in &policy_document.statements {
        writeln!(writer, r#"  statement {{"#)?;
        if let Some(ref sid) = statement.sid {
            writeln!(writer, r#"    sid = "{sid}""#)?;
        }
        writeln!(writer, r#"    effect = "{}""#, statement.effect)?;
        writeln!(writer, "    actions = {:?}", statement.actions)?;

        writeln!(
            writer,
            "    resources = {:?}",
            statement
                .resources
                .iter()
                .map(|s| replace_iam_interpolation(s))
                .collect::<Vec<_>>(),
        )?;
        for condition in &statement.conditions {
            writeln!(writer, "      condition {{")?;
            writeln!(writer, r#"      test = "{}""#, condition.test)?;
            writeln!(writer, r#"      variable = "{}""#, condition.variable)?;
            writeln!(
                writer,
                "      values = {:?}",
                condition
                    .values
                    .iter()
                    .map(|s| replace_iam_interpolation(s))
                    .collect::<Vec<_>>()
            )?;
            writeln!(writer, "      }}")?;
        }
        writeln!(writer, "  }}")?;
    }
    writeln!(writer, "}}")
}

fn replace_iam_interpolation(s: &str) -> String {
    // https://registry.terraform.io/providers/hashicorp/aws/latest/docs/data-sources/iam_policy_document#context-variable-interpolation
    s.replace("${", "&{")
}
