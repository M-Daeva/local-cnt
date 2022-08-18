#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:local-cnt";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let initial_state = State {
        owner: info.sender,
        count: msg.count,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(deps.storage, &initial_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", initial_state.owner.to_string())
        .add_attribute("count", initial_state.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => increment(deps, info),
        ExecuteMsg::Set { count } => set(deps, info, count),
    }
}

pub fn increment(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let storage = STATE.may_load(deps.storage)?;

    match storage {
        Some(mut state) => {
            if info.sender != state.owner {
                return Err(ContractError::CustomError {
                    val: "Sender is not owner!".to_string(),
                });
            }
            state.count += 1;
            STATE.save(deps.storage, &state)?;
            Ok(Response::new()
                .add_attribute("method", "increment")
                .add_attribute("owner", state.owner.to_string())
                .add_attribute("count", state.count.to_string()))
        }
        None => Err(ContractError::CustomError {
            val: "Can not get state!".to_string(),
        }),
    }
}

pub fn set(deps: DepsMut, info: MessageInfo, count: u8) -> Result<Response, ContractError> {
    let storage = STATE.may_load(deps.storage)?;

    match storage {
        Some(mut state) => {
            if info.sender != state.owner {
                return Err(ContractError::CustomError {
                    val: "Sender is not owner!".to_string(),
                });
            }
            state.count = count;
            STATE.save(deps.storage, &state)?;
            Ok(Response::new()
                .add_attribute("method", "set")
                .add_attribute("owner", state.owner.to_string())
                .add_attribute("count", state.count.to_string()))
        }
        None => Err(ContractError::CustomError {
            val: "Can not get state!".to_string(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => query_state(deps),
    }
}

pub fn query_state(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;

    to_binary(&CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::ContractError;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{attr, from_binary, Empty, Env, MessageInfo, OwnedDeps, Response};

    pub const ADDR1: &str = "addr1";
    pub const ADDR2: &str = "addr2";

    type Instance = (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        Env,
        MessageInfo,
        Result<Response, ContractError>,
    );

    fn get_instance(count: u8, addr: &str) -> Instance {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(addr, &[]);
        let msg = InstantiateMsg { count };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg);
        (deps, env, info, res)
    }

    #[test]
    fn test_init() {
        let (_, _, _, res) = get_instance(42, ADDR1);

        assert_eq!(
            res.unwrap().attributes,
            vec![
                attr("method", "instantiate"),
                attr("owner", ADDR1.to_string()),
                attr("count", "42")
            ]
        )
    }

    #[test]
    fn test_increment() {
        let (mut deps, env, info, _) = get_instance(42, ADDR1);
        let msg = ExecuteMsg::Increment {};
        let inc_res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            inc_res.unwrap().attributes,
            vec![
                attr("method", "increment"),
                attr("owner", ADDR1.to_string()),
                attr("count", "43")
            ]
        )
    }

    #[test]
    fn test_set() {
        let (mut deps, env, info, _) = get_instance(42, ADDR1);
        let msg = ExecuteMsg::Set { count: 45 };
        let set_res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(
            set_res.unwrap().attributes,
            vec![
                attr("method", "set"),
                attr("owner", ADDR1.to_string()),
                attr("count", "45")
            ]
        )
    }

    #[test]
    fn test_increment_wrong_addr() {
        let (mut deps, env, _, _) = get_instance(42, ADDR1);
        let msg = ExecuteMsg::Increment {};
        let info = mock_info(ADDR2, &[]);
        let res = execute(deps.as_mut(), env, info, msg);

        res.unwrap_err();
    }

    #[test]
    fn test_set_wrong_addr() {
        let (mut deps, env, _, _) = get_instance(42, ADDR1);
        let msg = ExecuteMsg::Set { count: 45 };
        let info = mock_info(ADDR2, &[]);
        let res = execute(deps.as_mut(), env, info, msg);

        res.unwrap_err();
    }

    #[test]
    fn test_query() {
        let (deps, env, _, _) = get_instance(42, ADDR1);
        let msg = QueryMsg::GetCount {};
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res = from_binary::<CountResponse>(&bin).unwrap();

        assert_eq!(res.count, 42);
    }
}
