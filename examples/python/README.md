# Python example

This simple example creates a small python script that asks the user for input and checks if the input is a prime number.

To build the project (i.e. generate code and documentation output), run the following command in the current directory:

```
> yarner
```

## Program structure

To create code in a certain files, use `file:<path/to/file>` as the block name.
First, we create a file `prime_numbers.py`:

```python
#- file:prime_numbers.py
# ==> Get user input.
# ==> Check if the input is a prime number.
```

## User input

We ask the user for input and convert it to an integer. To keep it short, we do not check for errors.

```python
#- Get user input
num = int(input("Enter a number: "))
```

## Prime number

We check if the input is a prime number and inform the user with the following code:

```python
#- Check if the input is a prime number
if num > 1:
   for i in range(2,num):
       if (num % i) == 0:
           print("%d is not a prime number" % (num,))
           print("%d times %d is %d" % (i, num//i, num))
           break
   else:
       print("%d is a prime number" % (num,))
else:
   print("%d is not a prime number" % (num,))
```
