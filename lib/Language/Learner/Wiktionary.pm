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
  join '', $start, $start->following_nodes->to_array->@*

}


9001;
