# ddnsclient
A simple DNS API client utility which allows its users to search/update/delete DNS records using the DNS provider API. Currently only supports [Mythic Beasts API](https://www.mythic-beasts.com/support/api/dnsv2).

The original requirement of the utility was to get **DDNS** but I decided to expand it and be fairly easy to implement new providers.

# Authentication

There are 2 ways to authenticate using this tool:
1. Passing user-pass when using the tool (supports environment variables as well - to prevent password from showing in the history)
2. Using a configuration file

These options are mutually exclusive. You will get error messages if you try to use both at the same time.

Regardless of the option you choose to go for I recommend adding an alias in your dotfiles to avoid typing the authentication options every time.

### 1. User-pass credentials
`./ddnsclient -u="your user" -p="your password" ZONE HOST`

### 2. Configuration file
`./ddnsclient --config="/path/to/dnsapiclient.config.yaml" ZONE HOST`

The configuration file allows you to set up API keys for different zones. The tool will pick up the key based on the arguments passed (ZONE-HOST-TYPE). See the [example.config.yaml](example.config.yaml) for more info.

# Usage

The client is intended to be simple and easy to use.

The main reason it was created was for giving users the ability to use **DDNS**. For this, run:

`ddnsclient ddns ZONE HOST`

## Search records
`ddnsclient [ZONE] [HOST] [TYPE]`

## Update records
`ddnsclient [ZONE] [HOST] [TYPE] update RECORDS`

Where RECORDS are provided as JSON. If RECORDS are not provided, application will read from stdin.

## Delete records
`ddnsclient delete [ZONE] [HOST] [TYPE]`

# Integrate with new providers
Under the **providers** folder, create a new module and name it after the provider. Then implement the **Provider** trait.
