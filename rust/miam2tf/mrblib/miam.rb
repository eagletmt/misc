Root = Struct.new(:users, :groups, :roles, :managed_policies, :instance_profiles) do
  def initialize
    super
    self.users ||= []
    self.groups ||= []
    self.roles ||= []
    self.managed_policies ||= []
    self.instance_profiles ||= []
  end
end

@root = Root.new
@context = Hashie::Mash.new(templates: {})

def user(name, path: nil, &block)
  user = User.new
  user.user_name = name
  user.path = path
  UserContext.new(user, context).instance_eval(&block)
  @root.users << user
end

def group(name, path: nil, &block)
  group = Group.new
  group.name = name
  group.path = path
  GroupContext.new(group, context).instance_eval(&block)
  @root.groups << group
end

def role(name, path: nil, &block)
  role = Role.new
  role.name = name
  role.path = path
  RoleContext.new(role, context).instance_eval(&block)
  @root.roles << role
end

def managed_policy(name, path: nil, &block)
  policy = ManagedPolicy.new
  policy.name = name
  policy.path = path
  raw = ManagedPolicyContext.new.instance_eval(&block)
  policy.policy_document = PolicyDocument.from_raw('ManagedPolicy', raw)
  @root.managed_policies << policy
end

def context
  @context
end

def template(name, &block)
  @context.templates[name] = block
end

def instance_profile(name, path: nil)
  @root.instance_profiles << InstanceProfile.new(name, path)
end

def exclude(_pattern)
  # miam2tf doesn't need to handle `exclude` method
end

User = Struct.new(:user_name, :path, :policies, :groups, :attached_managed_policies) do
  def initialize
    super
    self.policies ||= []
    self.groups ||= []
    self.attached_managed_policies ||= []
  end
end

module TemplateHelper
  def context
    @context
  end

  def include_template(template_name, context = {})
    block = @context.templates[template_name]
    saved = @context
    @context = @context.merge(context)
    instance_eval(&block)
    @context = saved
    nil
  end
end

class UserContext
  include TemplateHelper

  def initialize(user, context)
    @user = user
    @context = context
  end

  def policy(name, &block)
    @user.policies << PolicyDocument.from_raw(name, block.call)
  end

  def groups(*grps)
    @user.groups.concat(grps.map(&:to_s))
  end

  def attached_managed_policies(*policies)
    @user.attached_managed_policies.concat(policies.map(&:to_s))
  end
end

def json_principals_to_array(raw)
  list = []
  case raw
  when '*'
  when Hash
    raw.each do |type, identifiers|
        cond = PolicyPrincipal.new
        cond.type = type
        cond.identifiers = Array(identifiers).map(&:to_s)
        list << cond
    end
  end
  list
end

PolicyDocument = Struct.new(:name, :version, :statements) do
  def self.from_raw(name, raw)
    policy = new
    policy.name = name.to_s
    policy.version = raw['Version']
    statements = raw['Statement']
    unless statements.is_a?(Array)
      statements = [statements]
    end
    policy.statements = statements.map do |raw_stmt|
      stmt = PolicyStatement.new
      stmt.sid = raw_stmt['Sid']
      stmt.effect = raw_stmt['Effect']
      stmt.actions = Array(raw_stmt['Action'])
      stmt.resources = Array(raw_stmt['Resource'])
      stmt.conditions = []
      if raw_stmt.key?('Condition')
        raw_stmt['Condition'].each do |test, raw_cond|
          raw_cond.each do |variable, values|
            cond = PolicyCondition.new
            cond.test = test
            cond.variable = variable
            cond.values = Array(values).map(&:to_s)
            stmt.conditions << cond
          end
        end
      end
      stmt.principals = []
      if raw_stmt.key?('Principal')
        stmt.principals = json_principals_to_array(raw_stmt['Principal'])
      end

      stmt
    end
    policy
  end
end
PolicyStatement = Struct.new(:sid, :effect, :actions, :resources, :conditions, :principals)
PolicyCondition = Struct.new(:test, :variable, :values)
PolicyPrincipal = Struct.new(:identifiers, :type)

Group = Struct.new(:name, :path, :policies, :attached_managed_policies) do
  def initialize
    super
    self.policies ||= []
    self.attached_managed_policies ||= []
  end
end

class GroupContext
  include TemplateHelper

  def initialize(group, context)
    @group = group
    @context = context
  end

  def policy(name, &block)
    @group.policies << PolicyDocument.from_raw(name, block.call)
  end

  def attached_managed_policies(*policies)
    @group.attached_managed_policies.concat(policies.map(&:to_s))
  end
end

Role = Struct.new(:name, :path, :assume_role_policy_document, :policies, :attached_managed_policies, :instance_profiles, :max_session_duration) do
  def initialize
    super
    self.policies ||= []
    self.instance_profiles ||= []
    self.attached_managed_policies ||= []
  end
end

class RoleContext
  include TemplateHelper

  def initialize(role, context)
    @role = role
    @context = context
  end

  def assume_role_policy_document(&block)
    @role.assume_role_policy_document = PolicyDocument.from_raw('AssumeRolePolicyDocument', block.call)
  end

  def policy(name, &block)
    @role.policies << PolicyDocument.from_raw(name, block.call)
  end

  def attached_managed_policies(*policies)
    @role.attached_managed_policies.concat(policies.map(&:to_s))
  end

  def instance_profiles(*profiles)
    @role.instance_profiles.concat(profiles.map(&:to_s))
  end

  def max_session_duration(duration)
    @role.max_session_duration = duration.to_i
  end
end

ManagedPolicy = Struct.new(:name, :path, :policy_document)

class ManagedPolicyContext
end

InstanceProfile = Struct.new(:name, :path)
