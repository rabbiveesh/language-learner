use Language::Learner::Test types => 1;
use Language::Learner::Wiktionary;

use Mojo::DOM;


my $page = curfile->dirname->child(qw/pages feira/)->slurp;
my $wiki = Language::Learner::Wiktionary->new( dom => $page );

my @lines = split /\n/, $page;
my $ex_portuguese = Mojo::DOM->new(join "\n", @lines[167..230], '');


isa_ok $wiki->dom, ["Mojo::DOM"], 'the dom coerces';

is $wiki->language('Portuguese'), 
   $ex_portuguese->to_string, 'parsed correctly at end of file';



done_testing;
