package Language::Learner::Wiktionary;

use Language::Learner::Class;
use Mojo::DOM;
use Mojo::URL;
use Mojo::UserAgent;
use Pandoc;

my $ua = Mojo::UserAgent->new->max_redirects(5)
          ->with_roles('+Queued')->max_active(4);

my $base_url = Mojo::URL->new('https://en.wiktionary.org/wiki/');

#TODO- just put this elsewhere
sub _url_for ($query) {
  $base_url->clone->path($query)
}

has dom =>  (
  is => 'rw',
  isa => DOM,
  coerce => 1,
  required => 1,
  trigger => \&build_result
);

has language => (
  is => 'rw',
  isa => Str,
  trigger => \&build_result
);

has result => (
  is => 'rw',
  isa => MojoStr,
  coerce => 1
);

#TODO- implement grabbing like in other classes here

sub build_result ($self, $lang) {
  return unless $self->language;
  my $start = $self->dom->find("h2 span#$lang")->first->parent;
  my $for_lang = $start->following_nodes;
  unshift @$for_lang, $start;

  my $hr_found;
  $for_lang = $for_lang->map( sub {
    $hr_found++ if $_->matches('hr');
    return () if $hr_found;
    $_;
  });

  $self->result( $for_lang->join('') || 'nothing found' )
}

sub plaintext ($self) {
  my $html = $self->result;
  pandoc -f => 'html', -t => 'plain', { in => \$html, out => \my $md };
  $md
}

9001;
