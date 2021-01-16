import * as path from 'path';
import * as cdk from 'aws-cdk-lib';
import * as apigatewayv2 from 'aws-cdk-lib/aws-apigatewayv2';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import { LambdaProxyIntegration } from 'aws-cdk-lib/aws-apigatewayv2-integrations';

export class SlackSlashCommandSampleStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const handler = new lambda.DockerImageFunction(this, 'Handler', {
      code: lambda.DockerImageCode.fromImageAsset(path.join(__dirname, '../handler'), {
        repositoryName: 'slack-slash-command-sample',
      }),
      environment: {
        // FIXME: Should be stored into Secrets Manager or Parameter Store
        SLACK_SIGNING_SECRET: process.env.SLACK_SIGNING_SECRET!,
        SLACK_ACCESS_TOKEN: process.env.SLACK_ACCESS_TOKEN!,
      },
    });
    const httpApi = new apigatewayv2.HttpApi(this, 'HttpApi');
    httpApi.addRoutes({
      path: '/slash_command',
      methods: [apigatewayv2.HttpMethod.POST],
      integration: new LambdaProxyIntegration({
        handler,
      }),
    });
    httpApi.addRoutes({
      path: '/interactive',
      methods: [apigatewayv2.HttpMethod.POST],
      integration: new LambdaProxyIntegration({
        handler,
      }),
    });
    httpApi.addRoutes({
      path: '/external_select',
      methods: [apigatewayv2.HttpMethod.POST],
      integration: new LambdaProxyIntegration({
        handler,
      }),
    });
  }
}
