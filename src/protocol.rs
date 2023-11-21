use crate::party::party_client::PartyClientTrait;
use crate::party::party_server::PartyServerTrait;
use crate::party::TypeTrait;

struct NetworkServer {
    server: dyn PartyServerTrait,
    // TODO: Networking stuff
}

struct NetworkClient<T> where T: TypeTrait {
    client: dyn PartyClientTrait<T>,
    // TODO: Networking stuff
}

/// Executes the leaky kth ranked element protocol between the server and the given parties
/// and returns the found element.
pub fn leaky_kth_ranked_element<T, P, S>(server: &mut S, parties: &mut Vec<P>) -> Option<T>
    where
        T: TypeTrait,
        P: PartyClientTrait<T>,
        S: PartyServerTrait,
{
    let mut res = None;
    loop {
        let mut local_lt = vec![];
        let mut local_gt = vec![];
        // TODO: Here, we are iterating over all parties and collect there outputs (lt, gt) and then
        // input it to the server. Instead, they should all send it to the server over a connection.
        // Should also be done in parallel.
        for party in &mut *parties {
            let [lt, gt] = party.local_computation();
            local_lt.push(lt);
            local_gt.push(gt);
        }
        // TODO: Now, once the server has received all inputs from the parties (n parties), it should
        // broadcast sum_lt_enc and sum_gt_enc to all parties.
        let [sum_lt_enc, sum_gt_enc] = server.add_ciphertexts(&local_lt, &local_gt);
        let mut lt_shares = vec![];
        let mut gt_shares = vec![];

        // TODO: Here the same as above, this should be done in parallel and they should send their
        // shares (lt_sh, gt_sh) to the server.
        for party in &*parties {
            let [lt_sh, gt_sh] = party.compute_shares(sum_lt_enc.clone(), sum_gt_enc.clone());
            lt_shares.push(lt_sh);
            gt_shares.push(gt_sh);
        }

        // TODO: The "update" should be broadcasted to all parties.
        let sums = server.combine_shares(&lt_shares, &gt_shares);
        let update = server.calculate_update(sums);
        for party in &mut *parties {
            res = party.update_search_range(update);
            if res.is_some() {
                break;
            }
        }
        if res.is_some() {
            break;
        }
    }
    res
}