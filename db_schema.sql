DROP TABLE IF EXISTS SubstrateArchiver;
DROP TABLE IF EXISTS SubstrateArchiverBackfill;

CREATE TABLE IF NOT EXISTS SubstrateArchiver (
    Id INT AUTO_INCREMENT PRIMARY KEY,
    NetworkBlockId INT UNIQUE,
    WeaveVMArchiveTxid VARCHAR(66) UNIQUE
);

CREATE TABLE IF NOT EXISTS SubstrateArchiverBackfill (
    Id INT AUTO_INCREMENT PRIMARY KEY,
    NetworkBlockId INT UNIQUE,
    WeaveVMArchiveTxid VARCHAR(66) UNIQUE
);
