package Language::Learner::GoogleTranslate;
use Language::Learner::Class;
use Mojo::URL;
use Mojo::Promise;
use Mojo::UserAgent;
use List::MoreUtils qw/zip/;

#one ua for all instances
my $ua = Mojo::UserAgent->new->max_redirects(5)
          ->with_roles('+Queued')->max_active(4);

my $base_url = Mojo::URL->new('https://translate.google.com/translate_a/single');

#apparently, the dt option controls what gets returned by the
#query

my %parts = ( 
  translation      => 't',   #first field
  dictionary       => 'bd',  #second field
                             #3rd is translated language, for auto
  alt_translation  => 'at',  #sixth field
  definition       => 'md',
  examples         => 'ex',
  transliteration  => 'rm',
  corrections      => 'qca', #the eighth

  synonyms         => 'ss',  #the 11th
);

has language => (
  is => 'rw', 
  isa => Str,
  default => 'pt_BR'
);

has options => (
  is => 'rw',
  isa => ArrayRef,
  default => sub { [ qw/ corrections /] },
);


sub query_for ($self, $word) {
  $base_url->clone->query(
    client => 'gtx',
    ( map { ( dt => $parts{$_} ) } $self->options->@* ),
    sl => $self->language, tl => 'en', oe => 'utf-8', q => $word );
}

sub _make_request ($self, $word) {
  $ua->get_p($self->query_for($word))
     ->then(sub ($tx) { $self->_parse_response($tx->res) } )
     ->catch(sub { warn "lookup for $word failed: ", @_ , "\n" } )
     

}

sub _parse_response ($self, $res) {
  die join ' - ', $res->error->@{qw/code message/} if $res->error;
  my $json = $res->json;
  my $ret = { corrections => $json->[7][1] }
}

sub translate ($self, @targets) {
}
sub spell_check ($self, @targets) {
  my @promises;
  for (@targets) {
    push @promises, $self->_make_request($_);
  }
  my %corrected;
  Mojo::Promise->all(@promises)
    ->catch(sub { die "horrible death!: @_" } )
    ->then(sub (@results) {
        @results = map { $_->[0]{corrections} } @results;
        %corrected = ( zip @targets, @results );
      })->wait;

  $corrected{$_} ||= $_ for keys %corrected;
  if (@targets == 1) {
    return (values %corrected)[0]
  }
  return \%corrected
}







9991

