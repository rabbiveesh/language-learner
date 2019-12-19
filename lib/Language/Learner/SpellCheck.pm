package Language::Learner::SpellCheck;
use Language::Learner::Class;
use Text::Aspell;
use List::MoreUtils qw/pairwise/;
use Mojo::Util qw/slugify/;

has 'language' => (
  is => 'rw',
  isa => Str,
  default => 'pt_BR'
);

sub speller ($self) {
  my $spell = Text::Aspell->new;
  $spell->set_option('lang', $self->language);
  return $spell
}

sub check ($self, @list) {
  my $spell = $self->speller;

  my @ret;
  for (@list) {
    my @words = split ' ', $_;
    for my $word (@words) {
      next if $spell->check($word);
      my @suggests = map { decode 'UTF-8', $_ } $spell->suggest($word);
      @suggests = grep { $word eq slugify $_ } @suggests;
      next unless @suggests;
      $word = $suggests[0];
    }
    push @ret, join ' ', @words
  }

  return $ret[0] if @ret == 1;
  return { pairwise { ( $a => $b ) } @list, @ret }

}


10999
