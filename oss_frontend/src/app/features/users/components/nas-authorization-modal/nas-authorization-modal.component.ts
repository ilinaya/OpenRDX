import { Component, Inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { catchError, finalize } from 'rxjs/operators';
import { of } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatTableModule } from '@angular/material/table';
import { MatTabsModule } from '@angular/material/tabs';
import { NasService } from '../../../../shared/services/nas.service';
import { Nas } from '../../../../shared/models/nas.model';
import {MatError} from "@angular/material/select";

@Component({
  selector: 'app-nas-authorization-modal',
  templateUrl: './nas-authorization-modal.component.html',
  styleUrls: ['./nas-authorization-modal.component.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatDialogModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatTableModule,
    MatTabsModule,
    TranslateModule,
    MatError,
    MatTabsModule
  ]
})
export class NasAuthorizationModalComponent implements OnInit {
  loading = false;
  error: string | null = null;
  authorizedNas: Nas[] = [];
  availableNas: Nas[] = [];

  constructor(
    private dialogRef: MatDialogRef<NasAuthorizationModalComponent>,
    @Inject(MAT_DIALOG_DATA) public data: { userId: number; identifierId: number },
    private nasService: NasService,
    private translate: TranslateService
  ) {}

  ngOnInit(): void {
    this.loadData();
  }

  get hasAuthorizedNas(): boolean {
    return this.authorizedNas.length > 0;
  }

  get hasAvailableNas(): boolean {
    return this.availableNas.length > 0;
  }

  loadData(): void {
    this.loading = true;
    this.error = null;

    this.nasService.getAuthorizedNas(this.data.userId, this.data.identifierId).pipe(
      catchError(error => {
        this.error = this.translate.instant('users.detail.nasAuth.loadError');
        console.error('Error loading authorized NAS:', error);
        return of([]);
      }),
      finalize(() => {
        this.loading = false;
      })
    ).subscribe(nas => {
      this.authorizedNas = nas;
    });

    this.nasService.getAvailableNas(this.data.userId, this.data.identifierId).pipe(
      catchError(error => {
        this.error = this.translate.instant('users.detail.nasAuth.loadError');
        console.error('Error loading available NAS:', error);
        return of([]);
      })
    ).subscribe(nas => {
      this.availableNas = nas;
    });
  }

  authorizeNas(nas: Nas): void {
    this.loading = true;
    this.error = null;

    this.nasService.authorizeNas(this.data.userId, this.data.identifierId, nas.id).pipe(
      catchError(error => {
        this.error = this.translate.instant('users.detail.nasAuth.authorizeError');
        console.error('Error authorizing NAS:', error);
        return of(null);
      }),
      finalize(() => {
        this.loading = false;
      })
    ).subscribe(result => {
      if (result !== null) {
        this.loadData();
      }
    });
  }

  revokeAuthorization(nas: Nas): void {
    this.loading = true;
    this.error = null;

    this.nasService.revokeAuthorization(this.data.userId, this.data.identifierId, nas.id).pipe(
      catchError(error => {
        this.error = this.translate.instant('users.detail.nasAuth.revokeError');
        console.error('Error revoking authorization:', error);
        return of(null);
      }),
      finalize(() => {
        this.loading = false;
      })
    ).subscribe(result => {
      if (result !== null) {
        this.loadData();
      }
    });
  }

  authorizeAllNas(): void {
    if (confirm(this.translate.instant('users.detail.nasAuth.authorizeAllConfirm'))) {
      this.loading = true;
      this.error = null;

      this.nasService.authorizeAllNas(this.data.userId, this.data.identifierId).pipe(
        catchError(error => {
          this.error = this.translate.instant('users.detail.nasAuth.authorizeAllError');
          console.error('Error authorizing all NAS:', error);
          return of(null);
        }),
        finalize(() => {
          this.loading = false;
        })
      ).subscribe(result => {
        if (result !== null) {
          this.loadData();
        }
      });
    }
  }

  revokeAllAuthorizations(): void {
    if (confirm(this.translate.instant('users.detail.nasAuth.revokeAllConfirm'))) {
      this.loading = true;
      this.error = null;

      this.nasService.revokeAllAuthorizations(this.data.userId, this.data.identifierId).pipe(
        catchError(error => {
          this.error = this.translate.instant('users.detail.nasAuth.revokeAllError');
          console.error('Error revoking all authorizations:', error);
          return of(null);
        }),
        finalize(() => {
          this.loading = false;
        })
      ).subscribe(result => {
        if (result !== null) {
          this.loadData();
        }
      });
    }
  }
}
