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
  my $first = $self->dom->find("h2 span#$lang")->first;
  $first->parent->following_nodes
        ->join('')

}


9001;
