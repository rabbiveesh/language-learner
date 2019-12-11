package Language::Learner::GoogleImages;
use Language::Learner::Class;
use List::MoreUtils qw/pairwise/;
use Mojo::URL;
use Mojo::Promise;
use Mojo::UserAgent;
use Data::Printer;

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
    unshift @words, $options;
    $options = {};
  }

  my @promises;
  for my $word (@words) {
    my $query = $base_url->clone
      ->query(tbm => 'isch', hl => $self->language, q => $word);
    push @promises, $ua->get_p($query)->then(
      sub ($tx) {
        my $res = $tx->res;
        die join ' - ', $res->error->@{qw/code message/} if $res->error;

        my $results = $res->dom->find("img[alt*=$word]")
          ->map(sub { $_->parent->parent  }) #get the td
          ->map( sub { $_->at('a')->replace( $_->at('img') ) } ) 
          ->map( sub { 
            my $src = $_->at('img')->{src};
            my $text = $_->at('img')->remove->content;
            return { src => $src, text => $text }
          });
        return $results
    })->catch(sub { die "horrible death!: @_" } )
  }

  return Mojo::Promise->all(@promises)
    ->then(sub { my @res = map { $_->[0]->to_array } @_ } )
    ->then( sub (@results) {
      return [ pairwise {
          { word => $a, results => $b }
        } @words, @results ]
    })
    ->catch(sub { die "horrible death!: @_" } )

}

9089
