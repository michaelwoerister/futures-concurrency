use super::Join as JoinTrait;
use crate::utils::MaybeDone;

use core::fmt::{self, Debug};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project::pin_project;

macro_rules! impl_join_tuple {
    ($($F:ident)+) => (const _: () = {
        #[pin_project]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        #[allow(non_snake_case)]
        pub(super) struct Join<$($F: Future),*> {
            done: bool,
            $(#[pin] $F: MaybeDone<$F>,)*
        }

        impl<$($F),*> Debug for Join<$($F),*>
        where $(
            $F: Future + Debug,
            $F::Output: Debug,
        )* {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple("Join")
                    $(.field(&self.$F))*
                    .finish()
            }
        }

        #[async_trait::async_trait(?Send)]
        impl<$($F),*> JoinTrait for ($($F),*)
        where $(
            $F: IntoFuture,
        )* {
            type Output = ($($F::Output),*);
            async fn join(self) -> Self::Output {
                let ($($F),*): ($($F),*) = self;
                Join {
                    done: false,
                    $($F: MaybeDone::new($F.into_future())),*
                }.await
            }
        }

        impl<$($F: Future),*> Future for Join<$($F),*> {
            type Output = ($($F::Output),*);

            fn poll(
                self: Pin<&mut Self>, cx: &mut Context<'_>
            ) -> Poll<Self::Output> {
                let mut all_done = true;
                let mut this = self.project();
                assert!(!*this.done, "Futures must not be polled after completing");

                $(all_done &= this.$F.as_mut().poll(cx).is_ready();)*

                if all_done {
                    *this.done = true;
                    Poll::Ready(($(this.$F.take().unwrap()),*))
                } else {
                    Poll::Pending
                }
            }
        }
    }; )
}

impl_join_tuple! { A B }
impl_join_tuple! { A B C }
impl_join_tuple! { A B C D }
impl_join_tuple! { A B C D E }
impl_join_tuple! { A B C D E F }
impl_join_tuple! { A B C D E F G }
impl_join_tuple! { A B C D E F G H }
impl_join_tuple! { A B C D E F G H I }
impl_join_tuple! { A B C D E F G H I J }
impl_join_tuple! { A B C D E F G H I J K }
impl_join_tuple! { A B C D E F G H I J K L }
