/*
 * Copyright 2023 Signal Messenger, LLC
 * SPDX-License-Identifier: AGPL-3.0-only
 */

package org.signal.ringrtc;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import java.time.Instant;

public class CallLinkState {
  public enum Restrictions {
    NONE,
    ADMIN_APPROVAL,
    UNKNOWN,
  }

  /** Is never null, but may be empty. */
  @NonNull
  private final String name;
  @NonNull
  private final Restrictions restrictions;
  private final boolean revoked;
  @NonNull
  private final Instant expiration;
  @Nullable
  private final CallLinkEpoch epoch;

  /** Should only be used for testing. */
  public CallLinkState(@NonNull String name, @NonNull Restrictions restrictions, boolean revoked, @NonNull Instant expiration, @Nullable CallLinkEpoch epoch) {
    this.name = name;
    this.restrictions = restrictions;
    this.revoked = revoked;
    this.expiration = expiration;
    this.epoch = epoch;
  }

  @CalledByNative
  private CallLinkState(@NonNull String name, int rawRestrictions, boolean revoked, long expirationEpochSecond, @Nullable CallLinkEpoch epoch) {
    this.name = name;
    switch (rawRestrictions) {
    case 0:
      this.restrictions = Restrictions.NONE;
      break;
    case 1:
      this.restrictions = Restrictions.ADMIN_APPROVAL;
      break;
    default:
      this.restrictions = Restrictions.UNKNOWN;
    }
    this.revoked = revoked;
    this.expiration = Instant.ofEpochSecond(expirationEpochSecond);
    this.epoch = epoch;
  }

  /** Is never null, but may be empty. */
  @NonNull
  public String getName() {
    return name;
  }

  @NonNull
  public Restrictions getRestrictions() {
    return restrictions;
  }

  public boolean hasBeenRevoked() {
    return revoked;
  }

  @NonNull
  public Instant getExpiration() {
    return expiration;
  }

  @Nullable
  public CallLinkEpoch getEpoch() {
    return epoch;
  }
}
