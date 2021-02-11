Root = Struct.new(:users, :groups) do
  def initialize
    super
    self.users ||= []
    self.groups ||= []
  end
end

@root = Root.new

def user(name, path: nil, &block)
  user = User.new
  user.user_name = name
  user.path = path
  UserContext.new(user).instance_eval(&block)
  @root.users << user
end

def group(name, path: nil, &block)
  group = Group.new
  group.name = name
  group.path = path
  GroupContext.new(group).instance_eval(&block)
  @root.groups << group
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

class UserContext
  attr_reader :user

  def initialize(user)
    @user = user
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

  def include_template(template_name, context = {})
    # TODO
  end
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
            cond.values = Array(values)
            stmt.conditions << cond
          end
        end
      end
      stmt
    end
    policy
  end
end
PolicyStatement = Struct.new(:effect, :actions, :resources, :conditions)
PolicyCondition = Struct.new(:test, :variable, :values)

Group = Struct.new(:name, :path, :policies, :attached_managed_policies) do
  def initialize
    super
    self.policies ||= []
    self.attached_managed_policies ||= []
  end
end

class GroupContext
  attr_reader :group

  def initialize(group)
    @group = group
  end

  def policy(name, &block)
    @group.policies << PolicyDocument.from_raw(name, block.call)
  end

  def attached_managed_policies(*policies)
    @group.attached_managed_policies.concat(policies.map(&:to_s))
  end

  def include_template(template_name, context = {})
    # TODO
  end
end
