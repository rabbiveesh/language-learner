package Language::Learner::Common;

use Import::Into;

sub import {
  my $target = caller;

  Mojo::Util->import::into($target, qw/encode decode/);
  Mojo::File->import::into($target, qw/path curfile/);
  Try::Tiny->import::into($target);
  #Language::Leaner::Common::Exports->import::into($target);
  strict->import::into($target);
  warnings->import::into($target);
  feature->import::into($target, qw/:5.22 postderef postderef_qq signatures/);
  warnings->unimport::out_of($target, qw/experimental::postderef experimental::signatures/);
}

sub unimport {
 my $target = caller;

 feature->unimport::out_of($target, qw/:5.22 postderef postderef_qq signatures/);
 warnings->import::into($target, qw/experimental::postderef experimental::signatures/);
 #Language::Learner::Common::Exports->unimport::out_of($target);
 Try::Tiny->unimport::out_of($target);
 Mojo::Util->unimport::out_of($target, qw/encode decode/);
 Mojo::File->unimport::out_of($target, qw/path curfile/);
}

9003
