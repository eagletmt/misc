#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { SlackSlashCommandSampleStack } from '../lib/slack-slash-command-sample-stack';

const app = new cdk.App();
new SlackSlashCommandSampleStack(app, 'SlackSlashCommandSampleStack');
