#Language::Learner

This project is an attempt to automate my language learning
habits. I'm a huge fan of Fluent Forever, and I enjoy self
learning.

Currently, I'm learning Portuguese. Therefore, the tools are by
default looking for things in Portuguese. In the future, this will
be customizable with a config file.




#Layout

This package is arranged as follows. The main module is
Language::Learner. It is a frontend which ties together
individual tools, like Language::Learner::Wiktionary.
(The fact that I haven't done that yet is not indicative that this
isn't the plan, lol).

In addition to the tools, there are a bunch of Style Kits. This is
to keep the coding style and feature set the same across the whole
project. The main style is in Language::Learner::Common, which all
files should `use`. Then, there one for Tests, and one for
Classes.

I personally like using Types and Type Coercions, so those are
stored in Language::Learner::Types. All coercions and logic about
that should go there.

#Roadmap

I plan on adding the following tools:

-  google image search
-  google translate search
-  a spell checking search (to add/correct accents, maybe using
   google translate)
-  forvo.com search, to find audio

In addition, I plan on developing a frontend application for this,
so you can access all the tools from one screen.
