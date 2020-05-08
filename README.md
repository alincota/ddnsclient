# ddnsclient
A simple DDNS client which currently only supports Mythic Beasts API.

# Usage

The client is intended to be simple and easy to use.

The main reason it was created was for giving users the ability to use **DDNS**. For this, run:

`ddnsclient --username="your user" --password="your pass" ddns your.domain.com`

Note that in the future, a config file will be used instead to take the API credentials.

## Searching records
`ddnsclient --username="your user" --password="your pass" [ZONE] [HOST] [TYPE]`
