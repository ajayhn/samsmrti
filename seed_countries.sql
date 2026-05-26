-- Create Countries deck
INSERT OR IGNORE INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at)
VALUES ('dk_countries', 'Countries', NULL, 'World countries with capitals, rivers, mountains, cities, universities, and more', 20, 200, strftime('%s','now'), strftime('%s','now'));

-- Helper: each note needs notes row + cards for each template
-- Templates 0,6,7,8 = single card (ordinal as-is)
-- Templates 1,2,3,4,5 = each:Field, ordinal = tmpl*1000 + item_idx

-- ══════════════════════════════════════════════════════════════
-- 1. United States
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_us', 'dk_countries', 'nt_country', '{"Country":"United States","Capital":"Washington, D.C.","Rivers":"Mississippi, Colorado, Missouri, Ohio, Columbia","Languages":"English","Continent":"North America","Mountains":"Denali, Mount Rainier, Mount Whitney, Pikes Peak","Cities":"New York, Los Angeles, Chicago, Houston, Phoenix","Universities":"Harvard University, Massachusetts Institute of Technology, Stanford University, Yale University, Princeton University","Currency":"US Dollar (USD)","Flag":"🇺🇸"}', strftime('%s','now'), strftime('%s','now'));
-- single cards: Capital(0), Currency(6), Flag(7), Continent(8)
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_0','n_us',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_6','n_us',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_7','n_us',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_8','n_us',8,'new',strftime('%s','now'));
-- Rivers(1): 5 items
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_1_0','n_us',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_1_1','n_us',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_1_2','n_us',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_1_3','n_us',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_1_4','n_us',1004,'new',strftime('%s','now'));
-- Languages(2): 1
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_2_0','n_us',2000,'new',strftime('%s','now'));
-- Mountains(3): 4
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_3_0','n_us',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_3_1','n_us',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_3_2','n_us',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_3_3','n_us',3003,'new',strftime('%s','now'));
-- Cities(4): 5
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_4_0','n_us',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_4_1','n_us',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_4_2','n_us',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_4_3','n_us',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_4_4','n_us',4004,'new',strftime('%s','now'));
-- Universities(5): 5
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_5_0','n_us',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_5_1','n_us',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_5_2','n_us',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_5_3','n_us',5003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_us_5_4','n_us',5004,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 2. India
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_in', 'dk_countries', 'nt_country', '{"Country":"India","Capital":"New Delhi","Rivers":"Ganges, Yamuna, Brahmaputra, Godavari, Krishna","Languages":"Hindi, English, Tamil, Telugu, Bengali","Continent":"Asia","Mountains":"Kangchenjunga, Nanda Devi, Kamet, Annapurna","Cities":"Mumbai, Delhi, Bangalore, Chennai, Kolkata","Universities":"Indian Institute of Technology Bombay, Indian Institute of Science Bangalore, Jawaharlal Nehru University, University of Delhi, Indian Institute of Technology Delhi","Currency":"Indian Rupee (INR)","Flag":"🇮🇳"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_0','n_in',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_6','n_in',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_7','n_in',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_8','n_in',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_1_0','n_in',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_1_1','n_in',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_1_2','n_in',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_1_3','n_in',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_1_4','n_in',1004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_2_0','n_in',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_2_1','n_in',2001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_2_2','n_in',2002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_2_3','n_in',2003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_2_4','n_in',2004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_3_0','n_in',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_3_1','n_in',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_3_2','n_in',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_3_3','n_in',3003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_4_0','n_in',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_4_1','n_in',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_4_2','n_in',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_4_3','n_in',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_4_4','n_in',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_5_0','n_in',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_5_1','n_in',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_5_2','n_in',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_5_3','n_in',5003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_in_5_4','n_in',5004,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 3. China
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_cn', 'dk_countries', 'nt_country', '{"Country":"China","Capital":"Beijing","Rivers":"Yangtze, Yellow, Pearl, Mekong","Languages":"Mandarin","Continent":"Asia","Mountains":"Mount Everest, K2, Kunlun","Cities":"Shanghai, Beijing, Shenzhen, Guangzhou, Chengdu","Universities":"Tsinghua University, Peking University, Fudan University, Zhejiang University","Currency":"Yuan Renminbi (CNY)","Flag":"🇨🇳"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_0','n_cn',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_6','n_cn',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_7','n_cn',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_8','n_cn',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_1_0','n_cn',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_1_1','n_cn',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_1_2','n_cn',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_1_3','n_cn',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_2_0','n_cn',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_3_0','n_cn',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_3_1','n_cn',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_3_2','n_cn',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_4_0','n_cn',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_4_1','n_cn',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_4_2','n_cn',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_4_3','n_cn',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_4_4','n_cn',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_5_0','n_cn',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_5_1','n_cn',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_5_2','n_cn',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_cn_5_3','n_cn',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 4. United Kingdom
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_uk', 'dk_countries', 'nt_country', '{"Country":"United Kingdom","Capital":"London","Rivers":"Thames, Severn, Trent, Avon","Languages":"English, Welsh, Scottish Gaelic","Continent":"Europe","Mountains":"Ben Nevis, Snowdon, Scafell Pike","Cities":"London, Manchester, Birmingham, Edinburgh, Glasgow","Universities":"University of Oxford, University of Cambridge, Imperial College London, University College London, London School of Economics","Currency":"Pound Sterling (GBP)","Flag":"🇬🇧"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_0','n_uk',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_6','n_uk',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_7','n_uk',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_8','n_uk',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_1_0','n_uk',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_1_1','n_uk',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_1_2','n_uk',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_1_3','n_uk',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_2_0','n_uk',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_2_1','n_uk',2001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_2_2','n_uk',2002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_3_0','n_uk',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_3_1','n_uk',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_3_2','n_uk',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_4_0','n_uk',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_4_1','n_uk',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_4_2','n_uk',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_4_3','n_uk',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_4_4','n_uk',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_5_0','n_uk',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_5_1','n_uk',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_5_2','n_uk',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_5_3','n_uk',5003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_uk_5_4','n_uk',5004,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 5. France
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_fr', 'dk_countries', 'nt_country', '{"Country":"France","Capital":"Paris","Rivers":"Seine, Loire, Rhône, Garonne","Languages":"French","Continent":"Europe","Mountains":"Mont Blanc, Monte Cinto, Vignemale","Cities":"Paris, Marseille, Lyon, Toulouse, Nice","Universities":"Sorbonne University, École Polytechnique, École Normale Supérieure, Sciences Po","Currency":"Euro (EUR)","Flag":"🇫🇷"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_0','n_fr',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_6','n_fr',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_7','n_fr',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_8','n_fr',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_1_0','n_fr',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_1_1','n_fr',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_1_2','n_fr',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_1_3','n_fr',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_2_0','n_fr',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_3_0','n_fr',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_3_1','n_fr',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_3_2','n_fr',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_4_0','n_fr',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_4_1','n_fr',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_4_2','n_fr',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_4_3','n_fr',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_4_4','n_fr',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_5_0','n_fr',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_5_1','n_fr',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_5_2','n_fr',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_5_3','n_fr',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 6. Germany
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_de', 'dk_countries', 'nt_country', '{"Country":"Germany","Capital":"Berlin","Rivers":"Rhine, Danube, Elbe, Weser","Languages":"German","Continent":"Europe","Mountains":"Zugspitze, Watzmann, Feldberg","Cities":"Berlin, Munich, Hamburg, Frankfurt, Cologne","Universities":"Ludwig Maximilian University of Munich, Heidelberg University, Technical University of Munich, Humboldt University of Berlin","Currency":"Euro (EUR)","Flag":"🇩🇪"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_0','n_de',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_6','n_de',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_7','n_de',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_8','n_de',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_1_0','n_de',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_1_1','n_de',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_1_2','n_de',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_1_3','n_de',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_2_0','n_de',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_3_0','n_de',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_3_1','n_de',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_3_2','n_de',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_4_0','n_de',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_4_1','n_de',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_4_2','n_de',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_4_3','n_de',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_4_4','n_de',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_5_0','n_de',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_5_1','n_de',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_5_2','n_de',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_de_5_3','n_de',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 7. Japan
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_jp', 'dk_countries', 'nt_country', '{"Country":"Japan","Capital":"Tokyo","Rivers":"Shinano, Tone, Ishikari","Languages":"Japanese","Continent":"Asia","Mountains":"Mount Fuji, Mount Kita, Mount Hotaka","Cities":"Tokyo, Osaka, Yokohama, Nagoya, Kyoto","Universities":"University of Tokyo, Kyoto University, Osaka University, Waseda University","Currency":"Yen (JPY)","Flag":"🇯🇵"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_0','n_jp',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_6','n_jp',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_7','n_jp',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_8','n_jp',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_1_0','n_jp',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_1_1','n_jp',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_1_2','n_jp',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_2_0','n_jp',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_3_0','n_jp',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_3_1','n_jp',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_3_2','n_jp',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_4_0','n_jp',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_4_1','n_jp',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_4_2','n_jp',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_4_3','n_jp',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_4_4','n_jp',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_5_0','n_jp',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_5_1','n_jp',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_5_2','n_jp',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_jp_5_3','n_jp',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 8. Brazil
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_br', 'dk_countries', 'nt_country', '{"Country":"Brazil","Capital":"Brasília","Rivers":"Amazon, Paraná, São Francisco, Tocantins","Languages":"Portuguese","Continent":"South America","Mountains":"Pico da Neblina, Pico da Bandeira, Monte Roraima","Cities":"São Paulo, Rio de Janeiro, Brasília, Salvador, Fortaleza","Universities":"University of São Paulo, University of Campinas, Federal University of Rio de Janeiro, University of Brasília","Currency":"Real (BRL)","Flag":"🇧🇷"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_0','n_br',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_6','n_br',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_7','n_br',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_8','n_br',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_1_0','n_br',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_1_1','n_br',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_1_2','n_br',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_1_3','n_br',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_2_0','n_br',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_3_0','n_br',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_3_1','n_br',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_3_2','n_br',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_4_0','n_br',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_4_1','n_br',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_4_2','n_br',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_4_3','n_br',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_4_4','n_br',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_5_0','n_br',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_5_1','n_br',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_5_2','n_br',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_br_5_3','n_br',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 9. Australia
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_au', 'dk_countries', 'nt_country', '{"Country":"Australia","Capital":"Canberra","Rivers":"Murray, Darling, Murrumbidgee","Languages":"English","Continent":"Oceania","Mountains":"Mount Kosciuszko, Mount Townsend, Mount Bogong","Cities":"Sydney, Melbourne, Brisbane, Perth, Adelaide","Universities":"University of Melbourne, Australian National University, University of Sydney, University of New South Wales","Currency":"Australian Dollar (AUD)","Flag":"🇦🇺"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_0','n_au',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_6','n_au',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_7','n_au',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_8','n_au',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_1_0','n_au',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_1_1','n_au',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_1_2','n_au',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_2_0','n_au',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_3_0','n_au',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_3_1','n_au',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_3_2','n_au',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_4_0','n_au',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_4_1','n_au',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_4_2','n_au',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_4_3','n_au',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_4_4','n_au',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_5_0','n_au',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_5_1','n_au',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_5_2','n_au',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_au_5_3','n_au',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 10. Russia
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_ru', 'dk_countries', 'nt_country', '{"Country":"Russia","Capital":"Moscow","Rivers":"Volga, Ob, Yenisei, Lena, Amur","Languages":"Russian","Continent":"Europe/Asia","Mountains":"Mount Elbrus, Klyuchevskaya Sopka, Belukha","Cities":"Moscow, Saint Petersburg, Novosibirsk, Yekaterinburg, Kazan","Universities":"Lomonosov Moscow State University, Saint Petersburg State University, Moscow Institute of Physics and Technology, National Research University Higher School of Economics","Currency":"Ruble (RUB)","Flag":"🇷🇺"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_0','n_ru',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_6','n_ru',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_7','n_ru',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_8','n_ru',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_1_0','n_ru',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_1_1','n_ru',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_1_2','n_ru',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_1_3','n_ru',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_1_4','n_ru',1004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_2_0','n_ru',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_3_0','n_ru',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_3_1','n_ru',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_3_2','n_ru',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_4_0','n_ru',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_4_1','n_ru',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_4_2','n_ru',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_4_3','n_ru',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_4_4','n_ru',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_5_0','n_ru',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_5_1','n_ru',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_5_2','n_ru',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_ru_5_3','n_ru',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 11. South Korea
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_kr', 'dk_countries', 'nt_country', '{"Country":"South Korea","Capital":"Seoul","Rivers":"Han, Nakdong, Geum, Yeongsan","Languages":"Korean","Continent":"Asia","Mountains":"Hallasan, Jirisan, Seoraksan","Cities":"Seoul, Busan, Incheon, Daegu, Daejeon","Universities":"Seoul National University, Korea Advanced Institute of Science and Technology, Yonsei University, Korea University, Pohang University of Science and Technology","Currency":"Won (KRW)","Flag":"🇰🇷"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_0','n_kr',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_6','n_kr',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_7','n_kr',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_8','n_kr',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_1_0','n_kr',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_1_1','n_kr',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_1_2','n_kr',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_1_3','n_kr',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_2_0','n_kr',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_3_0','n_kr',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_3_1','n_kr',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_3_2','n_kr',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_4_0','n_kr',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_4_1','n_kr',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_4_2','n_kr',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_4_3','n_kr',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_4_4','n_kr',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_5_0','n_kr',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_5_1','n_kr',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_5_2','n_kr',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_5_3','n_kr',5003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_kr_5_4','n_kr',5004,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 12. Egypt
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_eg', 'dk_countries', 'nt_country', '{"Country":"Egypt","Capital":"Cairo","Rivers":"Nile","Languages":"Arabic","Continent":"Africa","Mountains":"Mount Catherine, Mount Sinai","Cities":"Cairo, Alexandria, Giza, Luxor, Aswan","Universities":"Cairo University, American University in Cairo, Al-Azhar","Currency":"Egyptian Pound (EGP)","Flag":"🇪🇬"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_0','n_eg',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_6','n_eg',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_7','n_eg',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_8','n_eg',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_1_0','n_eg',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_2_0','n_eg',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_3_0','n_eg',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_3_1','n_eg',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_4_0','n_eg',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_4_1','n_eg',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_4_2','n_eg',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_4_3','n_eg',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_4_4','n_eg',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_5_0','n_eg',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_5_1','n_eg',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_eg_5_2','n_eg',5002,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 13. Mexico
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_mx', 'dk_countries', 'nt_country', '{"Country":"Mexico","Capital":"Mexico City","Rivers":"Rio Grande, Grijalva, Usumacinta, Lerma","Languages":"Spanish","Continent":"North America","Mountains":"Pico de Orizaba, Popocatépetl, Iztaccíhuatl","Cities":"Mexico City, Guadalajara, Monterrey, Cancún, Puebla","Universities":"National Autonomous University of Mexico, Monterrey Institute of Technology and Higher Education, National Polytechnic Institute, Metropolitan Autonomous University","Currency":"Peso (MXN)","Flag":"🇲🇽"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_0','n_mx',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_6','n_mx',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_7','n_mx',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_8','n_mx',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_1_0','n_mx',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_1_1','n_mx',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_1_2','n_mx',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_1_3','n_mx',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_2_0','n_mx',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_3_0','n_mx',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_3_1','n_mx',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_3_2','n_mx',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_4_0','n_mx',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_4_1','n_mx',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_4_2','n_mx',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_4_3','n_mx',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_4_4','n_mx',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_5_0','n_mx',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_5_1','n_mx',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_5_2','n_mx',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_mx_5_3','n_mx',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 14. South Africa
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_za', 'dk_countries', 'nt_country', '{"Country":"South Africa","Capital":"Pretoria","Rivers":"Orange, Vaal, Limpopo","Languages":"English, Zulu, Xhosa, Afrikaans","Continent":"Africa","Mountains":"Mafadi, Thabana Ntlenyana, Table Mountain","Cities":"Johannesburg, Cape Town, Durban, Pretoria, Port Elizabeth","Universities":"University of Cape Town, University of the Witwatersrand, Stellenbosch University, University of Pretoria","Currency":"Rand (ZAR)","Flag":"🇿🇦"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_0','n_za',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_6','n_za',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_7','n_za',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_8','n_za',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_1_0','n_za',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_1_1','n_za',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_1_2','n_za',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_2_0','n_za',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_2_1','n_za',2001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_2_2','n_za',2002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_2_3','n_za',2003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_3_0','n_za',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_3_1','n_za',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_3_2','n_za',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_4_0','n_za',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_4_1','n_za',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_4_2','n_za',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_4_3','n_za',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_4_4','n_za',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_5_0','n_za',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_5_1','n_za',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_5_2','n_za',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_za_5_3','n_za',5003,'new',strftime('%s','now'));

-- ══════════════════════════════════════════════════════════════
-- 15. Italy
-- ══════════════════════════════════════════════════════════════
INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES
('n_it', 'dk_countries', 'nt_country', '{"Country":"Italy","Capital":"Rome","Rivers":"Po, Tiber, Arno, Adige","Languages":"Italian","Continent":"Europe","Mountains":"Mont Blanc, Monte Rosa, Matterhorn, Gran Paradiso","Cities":"Rome, Milan, Naples, Turin, Florence","Universities":"University of Bologna, Sapienza University of Rome, Politecnico di Milano, Bocconi University","Currency":"Euro (EUR)","Flag":"🇮🇹"}', strftime('%s','now'), strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_0','n_it',0,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_6','n_it',6,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_7','n_it',7,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_8','n_it',8,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_1_0','n_it',1000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_1_1','n_it',1001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_1_2','n_it',1002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_1_3','n_it',1003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_2_0','n_it',2000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_3_0','n_it',3000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_3_1','n_it',3001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_3_2','n_it',3002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_3_3','n_it',3003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_4_0','n_it',4000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_4_1','n_it',4001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_4_2','n_it',4002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_4_3','n_it',4003,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_4_4','n_it',4004,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_5_0','n_it',5000,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_5_1','n_it',5001,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_5_2','n_it',5002,'new',strftime('%s','now'));
INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_it_5_3','n_it',5003,'new',strftime('%s','now'));
