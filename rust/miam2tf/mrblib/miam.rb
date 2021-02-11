Root = Struct.new(:users) do
  def initialize
    super
    self.users ||= []
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

def exclude(_pattern)
  # miam2tf doesn't need to handle `exclude` method
end

User = Struct.new(:user_name, :path, :policies, :groups) do
  def initialize
    super
    self.policies ||= []
    self.groups ||= []
  end
end

class UserContext
  attr_reader :user

  def initialize(user)
    @user = user
  end

  def policy(name, &block)
    raw = block.call
    policy = PolicyDocument.new
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
    @user.policies << policy
  end

  def groups(*grps)
    @user.groups.concat(grps.map(&:to_s))
  end
end

PolicyDocument = Struct.new(:name, :version, :statements)
PolicyStatement = Struct.new(:effect, :actions, :resources, :conditions)
PolicyCondition = Struct.new(:test, :variable, :values)
