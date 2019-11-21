use Language::Learner::Test types => 1;
use Language::Learner::Wiktionary;

use Mojo::DOM;


my $page = curfile->dirname->child(qw/pages feira/)->slurp;
my $wiki = Language::Learner::Wiktionary->new( dom => $page );

my @lines = split /\n/, $page;
sub get_dom (@rows) { Mojo::DOM->new(join "\n", @lines[@rows], '' ) }
my $ex_portuguese = get_dom 167..230;
my $ex_galician   = get_dom 104..137;


isa_ok $wiki->dom, ["Mojo::DOM"], 'the dom coerces';

is $wiki->language('Portuguese'), 
   $ex_portuguese->to_string, 'parsed correctly at end of file';

is $wiki->language('Galician'),
   $ex_galician->to_string, 'parsed correctly in mid file';


done_testing;
