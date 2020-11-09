# K1 Assignment write-up

No, I didn't actually redo the assignment haha. This is just a quick writeup on why Rustdoc is awesome for doing assignment writeups.

One nice benefit of using Rust is that it has incredibly powerful built-in Markdown documentation support (Rustdoc). Aside from the output looking generally aesthetically alright, the "killer-feature" of doing writeups withing Rustdoc is the ability to directly link to types/code!

**NOTE:** Intra-doc links will not work when the page is exported/printed to a PDF! It would be a good to urge TAs to use the HTML output instead, as it would make their lives a lot easier (as they could review code directly in-browser).

For example, let's say I'm discussing the [`first_user_task`] function. Whoah, look at that, there's a link that takes you to the function! From that page, you can click on the `[src]` button in the top-right to jump directly to the function's implementation. Doc links even work across crate boundaries! e.g: here's a link to the [`choochoos::sys::create()`] syscall.

Note that this feature is limited to items which are in the same scope as the userspace crate (specifically, the `writeup` module). Notably, this excludes directly linking to `choochoos_kernel` code.

Linking to kernel code is possible, but it requires manually specifying the relative path to the item of interest. For example, here's a link to the [`Kernel::run()`](../../choochoos_kernel/kernel/struct.Kernel.html#method.run) implementation, which is implemented by writing <br> `[Kernel::run()](../../choochoos_kernel/kernel/struct.Kernel.html#method.run)`.

While this isn't quite full-blown [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming), using an in-code writeup can make a codebase much easier to understand and explore.

## Full Markdown Formatting is supported! 

You can _italicize_, **bold**, <sup>super</sup>, ~~cross things out~~, etc... to your heart's content!

Inline code samples also works.

```rust,ignore
fn test(foo: usize) -> Option<()> {
    None
}
```

And tables too:

Language | Number of CS 452 Kernels
---------|--------------------------
C/C++    | A lot
Rust     | Just 1

**TL;DR:** using Rustdoc for assignment writeups is pretty awesome :)
