# fatfs example

```
dfx start --clean
```

```
dfx deploy
dfx canister call fat ls
dfx canister call fat write_file '("hello.txt", "Hello, World!")'
dfx canister call fat ls
dfx canister call fat read_file '("hello.txt")'

dfx canister call fat write_file '("hello.txt", "Hello!")'
dfx canister call fat read_file '("hello.txt")'

dfx canister call fat write_file '("goodbye.txt", "Goodbye!")'
dfx canister call fat ls
dfx canister call fat read_file '("goodbye.txt")'
```
