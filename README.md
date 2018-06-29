# Lisaa

Lisaa is surely an acronym

The lisaa language is a simple scripting language featuring static typing.

# Usage

Just create a file in .lisaa and pass it as first argument

```
cargo build --release
target/release/lisaa my_file.lisaa
```


# Example

```
import string
fn main() {
     num a = 0;
     num b = 5;
     while b < 50 {
        b=b+1;
	    a = fac(b) + a;
	    a.toString().println();
    }
	return a;
}
fn fac(num a) -> num{
    if a == 1{
        return 1;
    }
    return a * fac(a-1);
}
```

Look at the source of string.lisaa to find more.

# Performance

The following benchmarks were done on the following code :
```
fn main(){
    // finds two factors of this number (241 and 307) 

    num toFind = 73987;
    num a = 0;
    num b = 0;
    num found = 0;
    while !found && a < toFind/2{
        a = a+1;
        b = 0;
        while !found && b < toFind/2{
            b = b + 1;
            if a * b == toFind{
                found = 1;
            }
        }
    }
}
```

| Language/interpreter | time  |
| -------------------- | --------- |
| lisaa (ast based interpreter) | 6000 ms |
| lisaa (using the vm) | 500 ms |
| rust | 7 ms |
| python | 1269 ms |
| java | 60 ms |

The Vm interpreting the bytecode is way faster than the original interpreter.

# Original interpreter

The original ast-based interpreter is now deprecated, it was too slow.
All the codes will now be compiled to be interpreted by the virtual machine

# The virtual machine

The code is compiled to bytecode interpreted by a stack machine.

The machine has a few instructions that manipulates the stack and can allocate memory on a heap.

# Garbage Collection

Coming soon

