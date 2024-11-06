# Rationale Behind Design Decisions

Instead of cluttering the code with comments, this document contains hashes
mentioned in a one line comment in the code. You can follow the hash here to
understand "why this code is written this way" or "why didn't she just.." etc.

- 6d12599a
    I am aware crossterm hands out resize events, and confirmed that it
    gives them out before/instead of key input events. *On my machine*
    I don't know how Windows, or other terminals and operating systems
    will react, or if this will ever change in the future. I'd rather
    the minimal overhead of re-checking the size every iteration than
    having the program panic because of breaking changes or edge cases.


