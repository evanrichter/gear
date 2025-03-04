// This file is part of Gear.

// Copyright (C) 2021-2022 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{manager::ExtManager, Config, Event, GasHandlerOf, Pallet, QueueOf};
use alloc::string::ToString;
use codec::Encode;
use common::{
    event::{MessageWokenSystemReason, SystemReason, UserMessageReadSystemReason},
    scheduler::*,
    storage::*,
    GasTree, Origin,
};
use core_processor::common::ExecutionErrorReason;
use gear_core::{
    ids::{CodeId, MessageId, ProgramId},
    message::ReplyMessage,
};

impl<T: Config> TaskHandler<T::AccountId> for ExtManager<T>
where
    T::AccountId: Origin,
{
    fn pause_program(&mut self, _program_id: ProgramId) {
        todo!("#646");
    }

    fn remove_code(&mut self, _code_id: CodeId) {
        todo!("#646");
    }

    // TODO: Generate system signal (instead of reply?) (issue #647).
    fn remove_from_mailbox(&mut self, user_id: T::AccountId, message_id: MessageId) {
        // Read reason.
        let reason = UserMessageReadSystemReason::OutOfRent.into_reason();

        // Reading message from mailbox.
        let mailboxed = Pallet::<T>::read_message(user_id, message_id, reason)
            .unwrap_or_else(|| unreachable!("Scheduling logic invalidated!"));

        // TODO: Consider sending here trap reply (issue #647).
        // Uncomment code below, once allowed split on cut node.
        // {
        //     // Generate trap reply.
        //     let trap = ExecutionErrorReason::OutOfRent.encode();

        //     // Creating reply message.
        //     let trap_reply = ReplyMessage::system(message_id, trap, core_processor::ERR_EXIT_CODE)
        //         .into_stored_dispatch(mailboxed.destination(), mailboxed.source(), message_id);

        //     // Splitting gas for newly created reply message.
        //     GasHandlerOf::<T>::split(message_id, trap_reply.id())
        //         .unwrap_or_else(|e| unreachable!("GasTree corrupted! {:?}", e));

        //     // Enqueueing dispatch into message queue.
        //     QueueOf::<T>::queue(trap_reply)
        //         .unwrap_or_else(|e| unreachable!("Message queue corrupted! {:?}", e));
        // }

        // Consuming gas handler for mailboxed message.
        Pallet::<T>::consume_message(mailboxed.id());
    }

    // TODO: Generate system signal (instead of reply?) (issue #647).
    fn remove_from_waitlist(&mut self, program_id: ProgramId, message_id: MessageId) {
        // Wake reason.
        let reason = MessageWokenSystemReason::OutOfRent.into_reason();

        // Waking dispatch.
        let waitlisted = Pallet::<T>::wake_dispatch(program_id, message_id, reason)
            .unwrap_or_else(|| unreachable!("Scheduling logic invalidated!"));

        // Trap explanation.
        let trap = ExecutionErrorReason::OutOfRent;

        // Generate trap reply.
        if self.check_program_id(&waitlisted.source()) {
            // Sending trap reply to program, by enqueuing it to message queue.
            let trap = trap.encode();

            // Creating reply message.
            let trap_reply = ReplyMessage::system(message_id, trap, core_processor::ERR_EXIT_CODE)
                .into_stored_dispatch(program_id, waitlisted.source(), message_id);

            // Splitting gas for newly created reply message.
            GasHandlerOf::<T>::split(trap_reply.id(), message_id)
                .unwrap_or_else(|e| unreachable!("GasTree corrupted! {:?}", e));

            // Enqueueing dispatch into message queue.
            QueueOf::<T>::queue(trap_reply)
                .unwrap_or_else(|e| unreachable!("Message queue corrupted! {:?}", e));
        } else {
            // Sending trap reply to user, by depositing event.
            //
            // There is no reason to use `Pallet::<T>::send_user_message( .. )`,
            // because there is no need in reply in future, so no reason
            // and funds to pay mailbox rent for it.

            // Note: for users, trap replies always contain
            // string explanation of the error.
            let trap = trap.to_string().into_bytes();

            // Creating reply message.
            let trap_reply = ReplyMessage::system(message_id, trap, core_processor::ERR_EXIT_CODE)
                .into_stored(program_id, waitlisted.source(), message_id);

            // Depositing appropriate event.
            Pallet::<T>::deposit_event(Event::UserMessageSent {
                message: trap_reply,
                expiration: None,
            });
        }

        // Consuming gas handler for waitlisted message.
        Pallet::<T>::consume_message(waitlisted.id());
    }

    fn remove_paused_program(&mut self, _program_id: ProgramId) {
        todo!("#646");
    }

    fn wake_message(&mut self, _program_id: ProgramId, _message_id: MessageId) {
        todo!("issue #349");
    }
}
