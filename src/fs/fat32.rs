#[repr(C, packed)]
struct DirectoryEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attrib: u8,
    userattrib: u8,
    undelete: u8,
    createtime: u16,
    createdate: u16,
    accessdate: u16,
    clusterhigh: u16,
    modifiedtime: u16,
    modifieddate: u16,
    clusterlow: u16,
    filesize: u32,
}

#[repr(C, packed)]
struct ClusterToLBA {
    FirstUsableCluster: u32,
    SectorsPerCluster: u32,
}

impl ClusterToLBA {
    fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.FirstUsableCluster + cluster * self.SectorsPerCluster - (2 * self.SectorsPerCluster)
    }
}
