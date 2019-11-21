package Language::Learner::Wiktionary;

use Language::Learner::Class;
use Mojo::DOM;

has dom =>  (
  is => 'rw',
  isa => DOM,
  coerce => 1,
  required => 1
);

sub language ($self, $lang) {
  my $start = $self->dom->find("h2 span#$lang")->first->parent;
  my $for_lang = $start->following_nodes;
  unshift @$for_lang, $start;

  my $hr_found;
  $for_lang = $for_lang->map( sub {
    $hr_found++ if $_->matches('hr');
    return () if $hr_found;
    $_;
  });

  $for_lang->join('')
}


9001;
