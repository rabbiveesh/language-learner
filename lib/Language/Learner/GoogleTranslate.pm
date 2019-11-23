package Lanugage::Learner::GoogleTranslate;
use Language::Learner::Class;
use Mojo::URL;


my $base_url = Mojo::URL->new('https://translate.google.com/translate_a/single');

#apparently, the dt option controls what gets returned by the
#query

my %parts = my @ ( 
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

