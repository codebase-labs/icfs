# fatfs example

```
dfx start --clean
```

```
dfx deploy

dfx canister call fatfs ls
dfx canister call fatfs write_file '("hello.txt", "Hello, World!")'
dfx canister call fatfs ls
dfx canister call fatfs read_file '("hello.txt")'

dfx canister call fatfs write_file '("hello.txt", "Hello!")'
dfx canister call fatfs read_file '("hello.txt")'

dfx canister call fatfs write_file '("goodbye.txt", "Goodbye!")'
dfx canister call fatfs ls
dfx canister call fatfs read_file '("goodbye.txt")'
```
