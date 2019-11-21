package Language::Learner::Test;

use Import::Into;

sub import {
  my %opts = @_;
  my $target = caller;
  Data::Printer->import::into($target);
  Test2::V0->import::into($target);
  Language::Learner::Types->import::into($target, -all) if $opts{types};
  #Language::Learner::Test::Exports->import::into($target);
  Language::Learner::Common->import::into($target);

}


1;
