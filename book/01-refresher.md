# Chapter I. A Refresher on Computer Concepts

[*Return to Index*](README.md)

It's difficult to write a tutorial that is approachable to beginners once you are familiar with a concept. You begin to forget what subjects were difficult to grasp and how unapproachable something can seem from the outside. On the flip side, you don't want to have to explain every minute detail and bore away those who remember the concepts. I hope this guide can strike a balance of both, being approachable to someone without any prior knowledge, while also being interesting to those who do.

Before we dive in deeper however, there are some fundamental concepts that everyone will need to be familiar with. While many of you will remember these from school -- especially for the types of people interested in reading this book -- there are those who might want a refresher. This first chapter is going to cover some introductory topics, such as binary, hexadecimal, bytes, and bitwise operations. If you feel comfortable with how all that works, feel free to skip ahead, there's nothing here you haven't seen before. For the rest of you, this should hopefully get you up to speed.

## What is binary?

For many of us, the subject of "bases" is covered in mathematics courses at some point in our teenage years. It is the idea that you can write the same number in different ways. In our day to day lives, we typically use "base 10", also known as "decimal". This means that we have ten different symbols used for writing digits, 0 through 9. After 9, we have no choice but to repeat symbols by "rolling over" the 9 back to 0 and putting a 1 in the ten's place, giving us "10". Decimal became the primary system for a variety of reasons (possibly because have ten fingers and toes), but it is not the only possible way to represent a number. If we instead had twelve different symbols to use before rolling over, that would be base 12. If you had only two symbols total, that would be base 2, or "binary".

Even in popular culture, the idea that computers use binary numbers in some fashion is pretty well known. Instead of having ten different symbols with which to use, binary only has two -- 0 and 1. Incrementing binary numbers quickly leads to more digits than their decimal counterparts, but they still represent the same value. If you have twelve apples sitting on a table, you could write that as "12" in decimal or "1100" in binary. It's two ways of representing the same value. While binary would would be more cumbersome in our daily lives, for computers binary is the more natural system. Digital computers deal with two options. Is something on or off? Is something true or false? Is there a voltage at some location or not? Is it 0 or 1? Electronic components throughout a computer can be represented with one of two binary choices. By chaining these circuits together, you can combine single binary digits, or "bits", into pairs of two, then groups of four, or eight, or so on.

## What is a byte?

To humans, there really isn't anything different about large or small numbers. Thinking about the number 100 is about the same as thinking about 1000. They're both abstract ideas. This is not the case for computers. Each number has to be stored as bits somewhere physically in the circuitry<sup>1</sup>. By chaining bits together, we can create higher and higher possible combinations. A single bit can only encode 0 or 1. Two bits though, can store 00, 01, 10, and 11 -- a range of four different numbers. With each bit added, the number of possibilities doubles. This seems great! With each additional bit we add, we have access to a larger and larger range of values. Unfortunately, there's a trade off. If you can only pack X number of bits total into a machine, you might be able to represent larger values, but you'll have fewer numbers available to work with. Eventually, the industry settled on eight bits being a pretty good standardized size, commonly referred to as a "byte". You get 256 different values, which is large enough for many uses, and mathematicians love that eight is a power of two. If your computer requires higher values, you can simply combine two bytes end-to-end to get a 16-bit value (which has 65536 possibilities), and you can repeat this method if you require more.

Contrary to popular opinion, referring to a machine as "8-bit", like the Game Boy, does not refer to its graphics. Instead, it refers to the size of data that the CPU handles. Most of the Game Boy's CPU functions deal with one byte of data (although as you'll see, there are some 16-bit operations, making it technically not a pure 8-bit system).

<sup>1</sup> Granted, the human brain also has to store things physically somewhere, but I'm an emulation developer not a neuroscientist.

## What is hexadecimal?

Instead of binary, you'll sometimes see computer data written in "hexadecimal", or base 16. There is a close relationship between binary and hexadecimal. Binary numbers can be written more compactly as hexadecimal, or "hex", while still retaining the same useful information. This relationship exists because sixteen is equal to 2<sup>4</sup>, meaning that four binary bits are exactly equal to a single hex digit. This is different than, say, decimal, where you would need four bits for all the decimal digits, with some possible binary values left over.

Here is an example of how this is done. As stated earlier, base 16 means there are sixteen different symbols available before needing to repeat. These are typically shown as 0 through 9, and rather than invent completely new squiggles to use, A through F are used for the other six. If we wished to count in decimal, binary, and hex, it would look like the following.

| Decimal | Binary | Hexadecimal |
| ------- | ------ | ----------- |
|    0    |  0000  |      0      |
|    1    |  0001  |      1      |
|    2    |  0010  |      2      |
|    3    |  0011  |      3      |
|    4    |  0100  |      4      |
|    5    |  0101  |      5      |
|    6    |  0110  |      6      |
|    7    |  0111  |      7      |
|    8    |  1000  |      8      |
|    9    |  1001  |      9      |
|   10    |  1010  |      A      |
|   11    |  1011  |      B      |
|   12    |  1100  |      C      |
|   13    |  1101  |      D      |
|   14    |  1110  |      E      |
|   15    |  1111  |      F      |
|   16    | 10000  |     10      |
|   17    | 10001  |     11      |
|   ...   |  ...   |     ...     |

As you can see, from zero to sixteen, a single hex value can represent the same values as four binary bits, with none left over. For this reason, rather than painstakingly write out every binary number, developers will instead replace every four bits with a hex digit, meaning that one byte can be written as two hex digits. Hexadecimal numbers are often prefixed with `0x` or `$` to distinguish from decimal (binary is likewise sometimes given a `0b` prefix). You'll see me use this notation throughout this book.

## Are there negative numbers in binary?

You'll notice I've only mentioned positive values thus far, yet Game Boy games would probably benefit from having negative numbers. Is that possible? Fortunately it is, utilizing a clever system. As we've established, computers must represent their numbers as some combination of binary bits, so the obvious solution might be to say that if the "most significant" (the left-most) bit is a 1, then that means it's negative. For example, 0010 is equal to 2, while 1010 would be equal to -2. This seems fine on the surface, but quickly presents a few problems. For one, what do you do if you need to add more bits? Typically, even in decimal, we act as if there are infinitely many leading zeroes to all our numbers, but that wouldn't work in this case. -2 would be equal to 1010 if we had four bits, but 10000010 if we have eight bits. Another issue is the bits 1000 would equal "-0", which doesn't exist in mathematics. Finally, operations like addition wouldn't work either. -2 + 2 should be zero, but instead 1010 + 0010 equals 1100 which is -4. This method is called "one's compliment", and as you can see, it's not widely used.

Instead, clever mathematicians created an alternative system known as "two's compliment", which solves all the problems listed above. Negative numbers do still have a leading 1, but getting a number's negative counterpart is done by taking the positive value's bits, flipping them all, then adding one. For example, we'll begin with the fact that 2 is 0010 in binary. To get -2, we would flip all the bits (1101), then add one, giving us 1110. This seems like a somewhat odd system, but it works quite well. All possible binary combinations are given their own unique representation, -0 is the same as 0 (flipping all the bits of 0000 gives 1111, and adding one gives 0000 again (remember that we have a finite number of bits, so if we carry too far, that value is just lost)), and addition works as you would expect: -2 + 2 = 0 and 1110 + 0010 = 0000.

The eagle-eyed among you might notice something odd though. Under this system, I said that 1110 was equal to -2, but in the previous section, 1110 was equal to 14. Why is there a difference? You'll soon see that when dealing with computer numbers, their context is essential. We might want a byte to always be positive and range from 0 to 255 (typically referred to as "unsigned"). Sometimes we might want negative values, where a single byte can instead range from -128 to 127 ("signed"). Sometimes we want a byte to correspond to a single text letter under a standardized system such as ASCII. Sometimes a byte represents audio or picture data. Sometimes a byte is actually eight true/false values compressed together for space. In these situations and more, the single byte can be written from 0x00 to 0xFF, but can represent entirely different things in the real world. You can't tell just by looking at the raw numbers, it's the context that makes the difference.

| Binary | Unsigned Value | Signed Value |
| ------ | -------------- | ------------ |
|  0000  |       0        |      0       |
|  0001  |       1        |      1       |
|  0010  |       2        |      2       |
|  0011  |       3        |      3       |
|  0100  |       4        |      4       |
|  0101  |       5        |      5       |
|  0110  |       6        |      6       |
|  0111  |       7        |      7       |
|  1000  |       8        |     -8       |
|  1001  |       9        |     -7       |
|  1010  |      10        |     -6       |
|  1011  |      11        |     -5       |
|  1100  |      12        |     -4       |
|  1101  |      13        |     -3       |
|  1110  |      14        |     -2       |
|  1111  |      15        |     -1       |

The length of the data also can impact its value. In the table above, the four bit value '1111' is equal to 15 when treated as an unsigned value, and -1 when it's signed. In decimal, we often ignore any proceeding zeroes when writing numbers, for example we will often write '9' instead of '09', and it still represents the same value. For unsigned values, this also applies. 1111 and 00001111 both represent 15. This is not the case for signed values -- 1111 represents -1 but 00001111 represents 15 again (an easy way to tell is that it starts with a 0, meaning it's positive). Here, the actual capacity of the value is impactful. This certainly can be confusing, but fortunately it's not something that will come up much. We will almost exclusively deal in bytes or multiple of bytes, and never truncate any leading zeroes.

## What operations can be done on binary numbers?

I performed addition between two binary numbers, and you can see that it works the same way as usual decimal addition. After all, they're two ways of writing the same value, we should always get the same result. The previous section also performed subtraction, which is just the addition of a negative number. Other common arithmetic operations like multiplication and division can be done, but they tend to be expensive to perform, and thus the Game Boy CPU doesn't support either. If you're planning on developing a Game Boy game and need multiplication, you'll have to add values in a loop.

"Shifting" all the bits either to the left or right is a common operation as well. For example, if we have a single byte with the value 10101010 and shifted left, we would move all the bits over one, discarding the left most bit (since a byte "must" be eight bits long), and filling the newly empty space with a zero, giving us 01010100. Left shifting has the nice effect of doubling any number, since the power of two each bit represented is now one higher. Similarly, right shifting halves a number. There's actually two different types of right shifts, however. "Logical" shift, where the empty space on the far left is replaced with a zero, and "arithmetic" shift, where the left-most bit is replaced with the same value it was before. This would be necessary to ensure negative numbers stay negative, for example. Theoretically, there would also be a corresponding logical and arithmetic left shift as well, but in practice you almost always only see logical left shifts (including with the Game Boy).

Aside from your typical mathematics, binary values also have several "bitwise operations", such as *and*, *or*, *exclusive or* (often abbreviated "xor"), and negation. These are called bitwise because they perform their operation on each bit individually. For example, if you *or* two bytes together, you compare each corresponding bit of the two numbers together. If either of them is a 1, then the result is a one, otherwise if both are 0, a zero. The *and* operation is a similar idea, but with a different result. Only if both compared bits are 1 is the result a one, otherwise it's a zero. *xor* is a one if both bits are different, otherwise it's a zero. Negation is perhaps the simplest. It operates on only one byte, but flips all the bits to be their opposite. If you found this to be a bit confusing, there are some cheat sheet "truth tables" that show the result of each operation when comparing two bits. The Game Boy has built in functionality for all of these, so we will be getting familiar with them.

Bitwise OR
| . | 0 | 1 |
| - | - | - |
| 0 | 0 | 1 |
| 1 | 1 | 1 |

Bitwise AND
| . | 0 | 1 |
| - | - | - |
| 0 | 0 | 0 |
| 1 | 0 | 1 |

Bitwise XOR
| . | 0 | 1 |
| - | - | - |
| 0 | 0 | 1 |
| 1 | 1 | 0 |

For example, let's say we have two bytes we want to *or* together -- 11110000 and 10100101. To complete this operation, we can go left to right, bit by corresponding bit between the two numbers, using our *or* table as defined above to determine the outcome. In this case, the left-most bits are both 1's, so the result is 1. Then a 1 *or* 0, yielding 1 again. 1 *or* 1 is 1. 1 *or* 0 is 1. 0 *or* 0 is 0, and so on. After the comparisons for all eight matching bits, we're left with the result 11110101.

The same two numbers can be used for *and* and *xor* operations, where instead a 1 is only given if both matching bits are a 1 or different, respectively. For our example numbers above, 11110000 *and* 10100101 gives 10100000, and 11110000 *xor* 10100101 gives 01010101.

## What does "endian" mean?

We discussed earlier that while we like to think of bytes as the standard size for numbers, it's common to require values larger than a single byte can hold<sup>2</sup>. By combining two or more bytes end-to-end, we get single values with a great deal more space. When we do this though, a question arises -- in which order do we write all the bytes? For example, if we have 0x12 and 0x34, should it be written 0x1234 or 0x3412?

![Image showing two different types of endianess](img/01-endianess.svg)

[By Aeroid - Own work, CC BY-SA 4.0](https://commons.wikimedia.org/w/index.php?curid=137790829)

The answer might seem obvious. It should be 0x1234, as that's order we read them. This is a perfectly valid solution, and it is known as "big endian". The opposite is also a common implementation, called "little endian", where the bytes are written least significant first. In our case, it's up to the CPU designers which they preferred, and for the Game Boy, they chose little endianness. There's some advantages to this. Yes, it's less intuitive to read, but most people, even back then, were using tools to manage data for them, rather than writing bytes in by hand. Little endian also makes dealing with values of different byte sizes easier. If you come across a value in little endian, you know that the first byte you see will be the smallest one. Subsequent bytes in the value (if any) are prepended from there. Like many conventions, it might sound tricky to keep track, but so long as you're consistent, everything works out neatly in the end.

<sup>2</sup> Shocking, I know.

[*Next Chapter*](02-cpu-specs.md)
