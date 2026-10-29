#include "var.h"
#include <stdint.h>
#include <stdio.h> // Preprocessor directive: include standard I/O declarations.
#include <stdlib.h>
#include <sys/types.h>

// Define a function that multiplies two integers.
int multiply(int a, int b) {
  int c;

  c = a * b;

  return c;
}

void normalCount(void) {
  int x = 0;

  printf("%d", x);

  x = x + 1;
}

void staticCount(void) {
  static int x = 0;

  printf("%d", x);

  x = x + 1;
}

// Convert lowercase letters to uppercase; assumes an ASCII-compatible encoding.
char toMyUpper(char c) {
  if (c >= 'a' && c <= 'z') {
    return c - ('a' - 'A'); // a = 97, A = 65, a - A = 32
  }

  return c;
}

enum MyNumbers {
  NumOne,
  NumTwo,
};

// Program entry point. The switch below returns before the remaining exercises.
int main(int argc, char *argv[]) {

  int myValue = 1;

  switch (myValue) {
  case 1:
    return 1;
  case 2:
    return 2;
  default:
    return 0;
  }

  double numbers[1000];

  double numbersTwo[3] = {1.5, 1.5, 1.5};

  int Month[3][3] = {
      {1, 2, 3},
      {1, 2, 3},
      {1, 2, 3},
  };

  printf("%f", numbersTwo[0]);

  // Declaration with an initializer: type, identifier, and initial value.
  double numberThree = 1.15;
  double a, b, c, d;

  double e, o = 1.5, x = 0.0, y;

  y = 5.5;

  int h, j, k;

  volatile int q = 10; // Volatile accesses must follow the implementation's volatile access rules; not thread synchronization.

  // Restrict exercise: if executed, the conflicting accesses below violate the restrict contract.

  int r = 10;

  int *restrict ptr = &r; // Restricted pointer to r.

  int *ptrTwo = &r; // Another pointer to the same object.

  *ptrTwo = 20;

  printf("%i", *ptr);

  // END

  extern int numOne;
  extern int numTwo;

  // register does not guarantee register storage; its address cannot be taken in C.
  register int numThree = 10;

  h = j = k = 10;

  int numberFour;

  int number1, number2, result;

  // Type Casting
  float myNumber = 1.0;

  int myNumberTwo = (int)myNumber; // Convert the floating-point value to int.

  char wordAlpha = 'A';
  char wordAlphaTwo = 65;

  int sumOne = (int)(1 + 2 + 3);         // The first step is addition
  int sumTwo = (int)1 + (int)2 + (int)3; // The first step is type casting

  // Type Conversion

  char *charA = "A";

  int numberCharA = atoi(charA);    // Parse a decimal integer string; "A" yields 0.
  int numberTwoCharA = atof(charA); // atof returns double (0.0 for "A"); assignment then converts it to int.

  printf("Value: %c\n", wordAlpha);

  printf("Multiply: ");

  scanf("%d %d", &number1, &number2);

  result = multiply(number1, number2);

  printf("Result: %d", result);

  // Automatic local state is initialized on each call.
  normalCount();
  normalCount();
  normalCount();

  // Static local state persists between calls.
  staticCount();
  staticCount();
  staticCount();

  char *charArray = "Kaan Bora Öz";

  toMyUpper('@');

  // Incomplete exercise: initialize i before evaluating the loop condition.
  for (int i; i < 1; i++) {
  }

  int num = 0;

  // Incomplete exercise: num never changes, so this loop would not terminate.
  while (num < 1) {
  }

  // This loop also needs an update to num to terminate.
  do {

  } while (num < 1);

  return 0;
}
/*
 * Application Workflow
 * Preprocessor Directives
 * Function Prototype OR Declarations
 */

/*
 * From c-study/: gcc -E src/main.c -o main.i (preprocess).
 * gcc -S main.i -o main.s (compile to assembly).
 * gcc -c main.s -o main.o (assemble).
 * gcc main.o -o main (link).
 */
