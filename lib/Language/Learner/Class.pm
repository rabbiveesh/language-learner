package Language::Learner::Class;

use Import::Into;

sub import {
  my $target = caller;
  Moo->import::into($target);
  MooX::late->import::into($target);
  Language::Learner::Types->import::into($target, -types);
  Language::Learner::Common->import::into($target);
}


1;
