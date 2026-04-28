#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String,
    panic_with_error, log,
};

// ============================================================
// ERROR CODES
// ============================================================
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TradePassError {
    NotInitialized     = 1,
    AlreadyInitialized = 2,
    Unauthorized       = 3,
    PassNotFound       = 4,
    PassAlreadyExists  = 5,
    PassRevoked        = 6,
}

// ============================================================
// DATA STRUCTURES
// ============================================================

/// Trạng thái của TradePass NFT
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PassStatus {
    Active   = 0,
    Revoked  = 1,
}

/// NFT TradePass - Soulbound (không thể chuyển nhượng)
#[contracttype]
#[derive(Clone, Debug)]
pub struct TradePass {
    pub owner:          Address,   // Địa chỉ ví của người dùng
    pub status:         PassStatus,// Trạng thái thẻ
    pub issued_at:      u64,       // Thời điểm cấp (Unix timestamp)
    pub revoked_at:     u64,       // Thời điểm thu hồi (0 nếu chưa)
    pub pass_id:        u64,       // ID định danh thẻ
    pub metadata_uri:   String,    // URI metadata (IPFS hoặc on-chain)
}

// ============================================================
// STORAGE KEYS
// ============================================================
#[contracttype]
pub enum DataKey {
    Admin,                  // Địa chỉ Admin
    PassCounter,            // Bộ đếm tổng số thẻ đã mint
    Pass(Address),          // TradePass theo địa chỉ ví
    Initialized,            // Trạng thái khởi tạo
}

// ============================================================
// CONTRACT
// ============================================================
#[contract]
pub struct TradePassContract;

#[contractimpl]
impl TradePassContract {

    // ==========================================================
    // 1. INIT - Khởi tạo hệ thống
    // ==========================================================

    /// Khởi tạo contract, thiết lập Admin
    /// Admin là bên kiểm duyệt và cấp thẻ duy nhất
    pub fn init(env: Env, admin: Address) {
        // Kiểm tra đã khởi tạo chưa
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, TradePassError::AlreadyInitialized);
        }

        // Admin phải ký xác nhận
        admin.require_auth();

        // Lưu Admin và khởi tạo bộ đếm
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PassCounter, &0u64);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Emit event khởi tạo
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "init"),),
            admin,
        );

        log!(&env, "TradePass: Contract initialized");
    }

    // ==========================================================
    // 2. MINT - Cấp NFT TradePass
    // ==========================================================

    /// Admin cấp TradePass NFT cho người dùng
    /// Thẻ là Soulbound - không thể chuyển nhượng
    /// Mỗi địa chỉ ví chỉ được cấp 1 thẻ duy nhất
    pub fn mint(
        env:          Env,
        admin:        Address,
        recipient:    Address,
        metadata_uri: String,
    ) -> u64 {
        // Xác thực hệ thống đã khởi tạo
        Self::assert_initialized(&env);

        // Xác thực caller là Admin
        admin.require_auth();
        Self::only_admin(&env, &admin);

        // Kiểm tra người dùng chưa có thẻ
        if env.storage().persistent().has(&DataKey::Pass(recipient.clone())) {
            panic_with_error!(&env, TradePassError::PassAlreadyExists);
        }

        // Lấy và tăng bộ đếm pass_id
        let pass_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PassCounter)
            .unwrap_or(0u64);
        let new_pass_id = pass_id + 1;

        // Tạo TradePass NFT mới
        let pass = TradePass {
            owner:        recipient.clone(),
            status:       PassStatus::Active,
            issued_at:    env.ledger().timestamp(),
            revoked_at:   0u64,
            pass_id:      new_pass_id,
            metadata_uri: metadata_uri.clone(),
        };

        // Lưu trữ
        env.storage()
            .persistent()
            .set(&DataKey::Pass(recipient.clone()), &pass);

        env.storage()
            .instance()
            .set(&DataKey::PassCounter, &new_pass_id);

        // Emit event mint
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "mint"),),
            (new_pass_id, recipient.clone(), metadata_uri),
        );

        log!(&env, "TradePass: Minted pass_id={}", new_pass_id);

        new_pass_id
    }

    // ==========================================================
    // 3. CHECK_PASS - Kiểm tra quyền giao dịch
    // ==========================================================

    /// Kiểm tra xem một địa chỉ ví có quyền giao dịch hợp lệ không
    /// Bất kỳ ai cũng có thể gọi hàm này
    /// Trả về true nếu thẻ tồn tại và đang Active
    pub fn check_pass(env: Env, user: Address) -> bool {
        Self::assert_initialized(&env);

        // Lấy thẻ của người dùng
        let pass: Option<TradePass> = env
            .storage()
            .persistent()
            .get(&DataKey::Pass(user.clone()));

        match pass {
            None => {
                // Không có thẻ
                log!(&env, "TradePass: No pass found for user");
                false
            }
            Some(p) => {
                // Kiểm tra trạng thái Active
                let is_valid = p.status == PassStatus::Active;
                log!(&env, "TradePass: check_pass result={}", is_valid);
                is_valid
            }
        }
    }

    /// Lấy toàn bộ thông tin TradePass của một địa chỉ
    pub fn get_pass(env: Env, user: Address) -> TradePass {
        Self::assert_initialized(&env);

        env.storage()
            .persistent()
            .get(&DataKey::Pass(user))
            .unwrap_or_else(|| panic_with_error!(&env, TradePassError::PassNotFound))
    }

    // ==========================================================
    // 4. REVOKE - Thu hồi thẻ
    // ==========================================================

    /// Admin thu hồi TradePass của người dùng vi phạm
    /// Thẻ bị revoke sẽ không còn quyền giao dịch
    /// Dữ liệu vẫn được giữ lại để audit trail
    pub fn revoke(env: Env, admin: Address, user: Address) {
        Self::assert_initialized(&env);

        // Xác thực Admin
        admin.require_auth();
        Self::only_admin(&env, &admin);

        // Lấy thẻ của người dùng
        let mut pass: TradePass = env
            .storage()
            .persistent()
            .get(&DataKey::Pass(user.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, TradePassError::PassNotFound));

        // Kiểm tra thẻ chưa bị thu hồi
        if pass.status == PassStatus::Revoked {
            panic_with_error!(&env, TradePassError::PassRevoked);
        }

        // Cập nhật trạng thái
        pass.status     = PassStatus::Revoked;
        pass.revoked_at = env.ledger().timestamp();

        // Lưu lại
        env.storage()
            .persistent()
            .set(&DataKey::Pass(user.clone()), &pass);

        // Emit event revoke
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "revoke"),),
            (pass.pass_id, user.clone()),
        );

        log!(&env, "TradePass: Revoked pass for user");
    }

    // ==========================================================
    // 5. ADMIN UTILS
    // ==========================================================

    /// Lấy địa chỉ Admin hiện tại
    pub fn get_admin(env: Env) -> Address {
        Self::assert_initialized(&env);
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, TradePassError::NotInitialized))
    }

    /// Lấy tổng số thẻ đã được mint
    pub fn total_passes(env: Env) -> u64 {
        Self::assert_initialized(&env);
        env.storage()
            .instance()
            .get(&DataKey::PassCounter)
            .unwrap_or(0u64)
    }

    /// Chuyển quyền Admin sang địa chỉ mới
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        Self::assert_initialized(&env);
        current_admin.require_auth();
        Self::only_admin(&env, &current_admin);

        env.storage()
            .instance()
            .set(&DataKey::Admin, &new_admin);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "admin_transfer"),),
            (current_admin, new_admin),
        );
    }

    // ==========================================================
    // INTERNAL HELPERS
    // ==========================================================

    /// Kiểm tra contract đã được khởi tạo chưa
    fn assert_initialized(env: &Env) {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, TradePassError::NotInitialized);
        }
    }

    /// Chỉ Admin mới được phép thực hiện
    fn only_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, TradePassError::NotInitialized));

        if &admin != caller {
            panic_with_error!(env, TradePassError::Unauthorized);
        }
    }
}

// ============================================================
// UNIT TESTS
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn setup() -> (Env, Address, TradePassContractClient<'static>) {
        let env    = Env::default();
        let admin  = Address::generate(&env);
        let contract_id = env.register_contract(None, TradePassContract);
        let client = TradePassContractClient::new(&env, &contract_id);
        (env, admin, client)
    }

    #[test]
    fn test_init() {
        let (env, admin, client) = setup();
        env.mock_all_auths();
        client.init(&admin);
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.total_passes(), 0);
    }

    #[test]
    fn test_mint_and_check() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);

        let user = Address::generate(&env);
        let uri  = String::from_str(&env, "ipfs://QmTradePassMetadata123");

        let pass_id = client.mint(&admin, &user, &uri);
        assert_eq!(pass_id, 1);
        assert_eq!(client.total_passes(), 1);

        // Kiểm tra quyền giao dịch
        let is_valid = client.check_pass(&user);
        assert!(is_valid, "User should have valid trading pass");

        // Lấy thông tin đầy đủ
        let pass = client.get_pass(&user);
        assert_eq!(pass.owner, user);
        assert_eq!(pass.status, PassStatus::Active);
        assert_eq!(pass.pass_id, 1);
    }

    #[test]
    fn test_revoke() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);

        let user = Address::generate(&env);
        let uri  = String::from_str(&env, "ipfs://QmTradePassMetadata123");

        client.mint(&admin, &user, &uri);

        // Trước khi revoke
        assert!(client.check_pass(&user));

        // Thu hồi thẻ
        client.revoke(&admin, &user);

        // Sau khi revoke
        assert!(!client.check_pass(&user), "Pass should be invalid after revoke");

        let pass = client.get_pass(&user);
        assert_eq!(pass.status, PassStatus::Revoked);
        assert!(pass.revoked_at > 0);
    }

    #[test]
    #[should_panic]
    fn test_double_mint_fails() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);
        let user = Address::generate(&env);
        let uri  = String::from_str(&env, "ipfs://QmTradePassMetadata123");

        client.mint(&admin, &user, &uri);
        client.mint(&admin, &user, &uri); // Phải panic
    }

    #[test]
    #[should_panic]
    fn test_unauthorized_mint_fails() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);

        let hacker = Address::generate(&env);
        let user   = Address::generate(&env);
        let uri    = String::from_str(&env, "ipfs://fake");

        // Hacker cố gắng mint thẻ - phải panic
        client.mint(&hacker, &user, &uri);
    }

    #[test]
    fn test_transfer_admin() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);

        let new_admin = Address::generate(&env);
        client.transfer_admin(&admin, &new_admin);

        assert_eq!(client.get_admin(), new_admin);
    }

    #[test]
    fn test_check_nonexistent_pass() {
        let (env, admin, client) = setup();
        env.mock_all_auths();

        client.init(&admin);

        let random_user = Address::generate(&env);
        // Người dùng không có thẻ -> false
        assert!(!client.check_pass(&random_user));
    }
}

