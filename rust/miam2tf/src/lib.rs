pub mod loader;
pub mod mruby;
mod mruby_c;
pub mod printer;

#[derive(Debug)]
pub struct Miam {
    pub users: Vec<User>,
    pub groups: Vec<Group>,
    pub roles: Vec<Role>,
    pub managed_policies: Vec<ManagedPolicy>,
    pub instance_profiles: Vec<InstanceProfile>,
}

#[derive(Debug)]
pub struct User {
    pub user_name: String,
    pub path: Option<String>,
    pub policies: Vec<PolicyDocument>,
    pub groups: Vec<String>,
    pub attached_managed_policies: Vec<String>,
}

#[derive(Debug)]
pub struct PolicyDocument {
    pub name: String,
    pub version: Option<String>,
    pub statements: Vec<PolicyStatement>,
}

#[derive(Debug)]
pub struct PolicyStatement {
    pub sid: Option<String>,
    pub effect: String,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub conditions: Vec<PolicyCondition>,

    pub principals: Vec<PolicyPrincipal>,

    pub not_actions: Vec<String>,
    pub not_resources: Vec<String>,
    pub not_principals: Vec<PolicyPrincipal>,
}

#[derive(Debug)]
pub struct PolicyCondition {
    pub test: String,
    pub variable: String,
    pub values: Vec<String>,
}

#[derive(Debug)]
pub struct PolicyPrincipal {
    pub typ: String,
    pub identifiers: Vec<String>,
}

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub path: Option<String>,
    pub policies: Vec<PolicyDocument>,
    pub attached_managed_policies: Vec<String>,
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub path: Option<String>,
    pub assume_role_policy_document: Option<PolicyDocument>,
    pub policies: Vec<PolicyDocument>,
    pub attached_managed_policies: Vec<String>,
    pub instance_profiles: Vec<String>,
    pub max_session_duration: Option<i64>,
}

#[derive(Debug)]
pub struct ManagedPolicy {
    pub name: String,
    pub path: Option<String>,
    pub policy_document: PolicyDocument,
}

#[derive(Debug)]
pub struct InstanceProfile {
    pub name: String,
    pub path: Option<String>,
}
