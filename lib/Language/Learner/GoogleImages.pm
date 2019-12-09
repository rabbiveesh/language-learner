package Language::Learner::GoogleImages;
use Language::Learner::Class;
use Mojo::URL;
use Mojo::Promise;
use Mojo::UserAgent;

my $ua = Mojo::UserAgent->new->max_redirects(5)
          ->with_roles('+Queued')->max_active(4);


my $base_url = Mojo::URL->new('https://www.google.com/search');

has language => (
  is => 'rw',
  isa => Str,
  default => 'pt-BR',
);

sub get_images ($self, $options, @words) {
  if ( Str->check($options) ) {
    unshift $options, @words;
    $options = {};
  }

  my @promises;
  for my $word (@words) {
    my $query = $base_url->clone->query(tbm => 'isch', hl => $self->language, q => $word);
    push @promises, $ua->get_p($query)
  }

  return Mojo::Promise->all(@promises)->then(sub { map { $_->[0]->res } @_ } )

}
