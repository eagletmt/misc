#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import { SlackSlashCommandSampleStack } from '../lib/slack-slash-command-sample-stack';

const app = new cdk.App();
new SlackSlashCommandSampleStack(app, 'SlackSlashCommandSampleStack');
