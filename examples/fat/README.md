# fatfs example

```
dfx start --clean
dfx deploy
dfx canister call fat write_hello '("World")'
dfx canister call fat ls
dfx canister call fat read_hello
```
