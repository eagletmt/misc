pub fn load_miam<P>(path: P) -> Result<crate::Miam, anyhow::Error>
where
    P: AsRef<std::path::Path>,
{
    let mruby = crate::mruby::MRuby::default();
    mruby.load(path)?;
    let root = mruby.instance_variable_get("@root");
    to_miam(&root)
}

fn to_rust_policy_document(policy: &crate::mruby::Value) -> crate::PolicyDocument {
    let name = policy.read_attribute("name").to_string();
    let version = policy.read_attribute("version").to_string_opt();
    let mut statements = Vec::new();
    for statement in policy.read_attribute("statements").iter() {
        let sid_value = statement.read_attribute("sid");
        let sid = if sid_value.is_nil() {
            None
        } else {
            Some(sid_value.to_string())
        };

        let effect = statement.read_attribute("effect").to_string();
        let mut actions = Vec::new();
        for action in statement.read_attribute("actions").iter() {
            actions.push(action.to_string());
        }
        let mut resources = Vec::new();
        for resource in statement.read_attribute("resources").iter() {
            resources.push(resource.to_string());
        }
        let mut conditions = Vec::new();
        for condition in statement.read_attribute("conditions").iter() {
            let test = condition.read_attribute("test").to_string();
            let variable = condition.read_attribute("variable").to_string();
            let mut values = Vec::new();
            for value in condition.read_attribute("values").iter() {
                values.push(value.to_string());
            }
            conditions.push(crate::PolicyCondition {
                test,
                variable,
                values,
            });
        }
        let mut principals = Vec::new();
        for principal in statement.read_attribute("principals").iter() {
            let typ = principal.read_attribute("type").to_string();
            let mut identifiers = Vec::new();
            for identifier in principal.read_attribute("identifiers").iter() {
                identifiers.push(identifier.to_string());
            }
            principals.push(crate::PolicyPrincipal { typ, identifiers });
        }
        statements.push(crate::PolicyStatement {
            sid,
            effect,
            actions,
            resources,
            conditions,
            principals,
        });
    }
    crate::PolicyDocument {
        name,
        version,
        statements,
    }
}

fn to_miam(root: &crate::mruby::Value) -> Result<crate::Miam, anyhow::Error> {
    let mut users = Vec::new();
    for user in root.read_attribute("users").iter() {
        let user_name = user.read_attribute("user_name").to_string();
        let path = user.read_attribute("path").to_string_opt();
        let mut policies = Vec::new();
        for policy in user.read_attribute("policies").iter() {
            policies.push(to_rust_policy_document(&policy));
        }
        let mut groups = Vec::new();
        for group in user.read_attribute("groups").iter() {
            groups.push(group.to_string());
        }
        let mut attached_managed_policies = Vec::new();
        for policy in user.read_attribute("attached_managed_policies").iter() {
            attached_managed_policies.push(policy.to_string());
        }
        users.push(crate::User {
            user_name,
            path,
            policies,
            groups,
            attached_managed_policies,
        });
    }

    let mut groups = Vec::new();
    for group in root.read_attribute("groups").iter() {
        let name = group.read_attribute("name").to_string();
        let path = group.read_attribute("path").to_string_opt();
        let mut policies = Vec::new();
        for policy in group.read_attribute("policies").iter() {
            policies.push(to_rust_policy_document(&policy));
        }
        let mut attached_managed_policies = Vec::new();
        for policy in group.read_attribute("attached_managed_policies").iter() {
            attached_managed_policies.push(policy.to_string());
        }
        groups.push(crate::Group {
            name,
            path,
            policies,
            attached_managed_policies,
        });
    }
    let mut roles = Vec::new();
    for role in root.read_attribute("roles").iter() {
        let name = role.read_attribute("name").to_string();
        let path = role.read_attribute("path").to_string_opt();
        let assume_role_policy_document = role.read_attribute("assume_role_policy_document");
        let assume_role_policy_document = if assume_role_policy_document.is_nil() {
            None
        } else {
            Some(to_rust_policy_document(&assume_role_policy_document))
        };
        let mut policies = Vec::new();
        for policy in role.read_attribute("policies").iter() {
            policies.push(to_rust_policy_document(&policy));
        }
        let mut attached_managed_policies = Vec::new();
        for policy in role.read_attribute("attached_managed_policies").iter() {
            attached_managed_policies.push(policy.to_string());
        }
        let mut instance_profiles = Vec::new();
        for profile in role.read_attribute("instance_profiles").iter() {
            instance_profiles.push(profile.to_string());
        }
        let max_session_duration = role.read_attribute("max_session_duration").to_i64_opt();
        roles.push(crate::Role {
            name,
            path,
            assume_role_policy_document,
            policies,
            attached_managed_policies,
            max_session_duration,
            instance_profiles,
        });
    }

    let mut managed_policies = Vec::new();
    for policy in root.read_attribute("managed_policies").iter() {
        let name = policy.read_attribute("name").to_string();
        let path = policy.read_attribute("path").to_string_opt();
        let policy_document = to_rust_policy_document(&policy.read_attribute("policy_document"));
        managed_policies.push(crate::ManagedPolicy {
            name,
            path,
            policy_document,
        });
    }

    let mut instance_profiles = Vec::new();
    for profile in root.read_attribute("instance_profiles").iter() {
        let name = profile.read_attribute("name").to_string();
        let path = profile.read_attribute("path").to_string_opt();
        instance_profiles.push(crate::InstanceProfile { name, path });
    }

    Ok(crate::Miam {
        users,
        groups,
        roles,
        managed_policies,
        instance_profiles,
    })
}
