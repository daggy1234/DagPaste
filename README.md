# DagPaste

its a cool pastebin idk

## Static

Here are all the static files:
https://mega.nz/folder/QZ8Clbia#kCmJm3iS4PkwYHuKBRzT5g




## Run

NGINX config exists. Add your SSL certs in the NGINX certs dir. Also maybe edit the NGINX name to your domain.

I'll host the static directory somewhere else, as they kinda are very CSS/HTML heavy.

otherwise everything else is ready to go. 

Just use 
```shell
docker-compose up -d
```

and there ya go.

## Development

Remember to use nightly for Rocket to compile.

Otherwise again download static dir.

Simply run 

```
cargo run
```

for development.