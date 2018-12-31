There's not much going on yet.
The initial implementation is there.

It currently only supports going from raw mutf8 to utf8, but it's a first step...

There's a couple of goals that I'd want to have done before I count this crate as "ok".

* [ ] Support some conversions from utf8 to mutf8.
* [ ] Add more functionality to try to bring it into line with a normal String/str impl.


I'll no doubt end up revisiting this list, or even forgetting something as I work on it, so this isn't really a feature list.
More of a "What I don't forget to write down somewhere".
I've got a couple of things I want to do with this crate, but the first goal would be to try to find out how I would want to use it.

I have another crate `class_file`.
It will end up using this crate extensively, and I want to shape this crate to aid that.

As a second priority, yes, I would want this crate to be the "goto" crate when dealing with mutf8.
