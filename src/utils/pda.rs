use solana_program::pubkey::Pubkey;

pub fn get_vanilla_token_mint(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
) -> (Pubkey, u8, Vec<Vec<u8>>) {
    let (addr, bump) =
        Pubkey::find_program_address(&[b"vanilla", token_2022_mint.as_ref()], &program_id);

    let seeds = vec![
        b"vanilla".to_vec(),
        token_2022_mint.as_ref().to_vec(),
        vec![bump],
    ];

    (addr, bump, seeds)
}

pub fn get_reserve_authority(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
) -> (Pubkey, u8, Vec<Vec<u8>>) {
    let (addr, bump) =
        Pubkey::find_program_address(&[b"reserve_authority", token_2022_mint.as_ref()], &program_id);

    let seeds = vec![
        b"reserve_authority".to_vec(),
        token_2022_mint.as_ref().to_vec(),
        vec![bump],
    ];

    (addr, bump, seeds)
}

pub fn get_token_mint_authority(
    token_mint: Pubkey,
    program_id: Pubkey,
) -> (Pubkey, u8, Vec<Vec<u8>>) {
    let (addr, bump) =
        Pubkey::find_program_address(&[b"mint_authority", token_mint.as_ref()], &program_id);

    let seeds: Vec<Vec<u8>> = vec![
        b"mint_authority".to_vec(),
        token_mint.as_ref().to_vec(),
        vec![bump],
    ];

    (addr, bump, seeds)
}

pub fn get_token_freeze_authority(
    token_mint: Pubkey,
    program_id: Pubkey,
) -> (Pubkey, u8, Vec<Vec<u8>>) {
    let (addr, bump) =
        Pubkey::find_program_address(&[b"freeze_authority", token_mint.as_ref()], &program_id);

    let seeds: Vec<Vec<u8>> = vec![
        b"freeze_authority".to_vec(),
        token_mint.as_ref().to_vec(),
        vec![bump],
    ];

    (addr, bump, seeds)
}