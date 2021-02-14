# envwarden
Set environment variables from Bitwarden Secure Notes.

## Prerequisites
- Create a folder for envwarden in Bitwarden
- Save values as custom fields of Secure Note in the folder
    - The field name is mapped to environment variable name
    - The field value is mapped to environment variable value
    - The field type must be "Text" or "Hidden"
- Install Bitwarden CLI
    - https://github.com/bitwarden/cli

## Usage
```
% export BW_SESSION=...
% export ENVWARDEN_FOLDERID=$(bw get folder envwarden | jq -r .id)
% envwarden aws printenv AWS_ACCESS_KEY_ID
AKIA................
```
